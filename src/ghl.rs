//! Authentic GHL v2 house policy. Internal lanes: OPEN=0, W1..W3=1..3, B1..B3=4..6.
use crate::{engine::{Gem,Note,Options,Score,Track},pitched::Finding,timing::Timeline};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Clone)]
struct Event{tick:i64,notes:Vec<Note>,pitches:Vec<i32>,lanes:Vec<i32>}
const TEMPLATES:&[&[i32]]=&[&[1,2,3],&[4,5,6],&[1,2,3,6],&[1,4,2,5,3,6],&[1,5,3],&[1,0,2,3],&[1,4,2,5],&[1,2,5,6],&[4,1,5,2,6,3],&[4,2,6],&[4,5,2,3],&[4,1,5,2]];
pub fn encode(lane:i32)->i32{match lane{0=>7,1=>0,2=>1,3=>2,4=>3,5=>4,6=>8,_=>unreachable!()}}
pub fn preview_lane(lane:i32)->Option<usize>{match lane{0=>Some(0),1=>Some(1),2=>Some(2),3=>Some(3),4=>Some(4),8=>Some(5),7=>Some(6),_=>None}}
fn column(l:i32)->i32{if l==0{-1}else{(l-1)%3}}
fn row(l:i32)->i32{if l==0{-1}else{(l-1)/3}}
fn center(l:&[i32])->f64{l.iter().sum::<i32>() as f64/l.len().max(1) as f64}
fn group(track:&Track)->Vec<Event>{let mut groups:BTreeMap<i64,Vec<Note>>=BTreeMap::new();for n in &track.notes{groups.entry(n.tick.round() as i64).or_default().push(n.clone());}groups.into_iter().map(|(tick,notes)|{let pitches=notes.iter().map(|n|n.pitch).collect::<BTreeSet<_>>().into_iter().collect();Event{tick,notes,pitches,lanes:vec![]}}).collect()}
fn phrases(e:&[Event],score:&Score)->Vec<std::ops::Range<usize>>{
 let mut out=vec![];let mut first=0;let bar=crate::engine::bar_ticks(score);
 for i in 1..e.len(){let prior=&e[i-1];let release=prior.notes.iter().map(|n|prior.tick+n.length.round() as i64).max().unwrap_or(prior.tick);let legato=e[i].notes.iter().any(|n|n.hopo)||release>e[i].tick;
 let section=score.sections.iter().any(|s|s.tick>prior.tick as f64&&s.tick<=e[i].tick as f64);
 let rest=e[i].tick-release.max(prior.tick)>=score.ppq;
 // Recognize repeated rhythmic/pitch blocks at measure boundaries before fallback groups.
 let count=i-first;let repeat=count>=2&&i+count<=e.len()&&e[i].tick%bar==0&&(0..count).all(|j|e[first+j].pitches==e[i+j].pitches&&e[first+j].tick-e[first].tick==e[i+j].tick-e[i].tick);
 if section||rest||!legato&&(repeat||e[i].tick-e[first].tick>=bar*4){out.push(first..i);first=i;}
 }if first<e.len(){out.push(first..e.len());}out
}
// Include timing, individual releases and articulation in transposition-invariant family keys.
type Fingerprint=Vec<(i64,Vec<(i32,i64,u16)>)>;
fn fingerprint(e:&[Event])->Fingerprint{let root=e[0].pitches[0];let start=e[0].tick;e.iter().map(|event|(event.tick-start,event.notes.iter().map(|n|(n.pitch-root,n.length.round() as i64,flags(n))).collect())).collect()}
fn flags(n:&Note)->u16{n.hopo as u16|(n.tap as u16)<<1|(n.palm_mute as u16)<<2|(n.dead as u16)<<3|(n.staccato as u16)<<4|(n.picked as u16)<<5|(n.tremolo as u16)<<6|(n.slide as u16)<<7|(n.bend as u16)<<8}
fn pedal(e:&[Event])->Option<i32>{let mut counts=BTreeMap::new();for event in e.iter().filter(|e|e.pitches.len()==1){*counts.entry(event.pitches[0]).or_insert(0usize)+=1;}let (&low,&count)=counts.iter().next()?;let qualifies=count>=3&&count as f64/e.len() as f64>=0.3&&e.iter().any(|v|v.pitches[0]>low)||count>=3&&count==e.len();qualifies.then_some(low)}
fn open(event:&Event,pedal:Option<i32>,options:&Options)->bool{options.ghl_open_pedals&&event.pitches.len()==1&&Some(event.pitches[0])==pedal&&event.notes.iter().all(|n|(n.picked||!n.hopo&&!n.tap)&&!n.tap&&!n.hopo)}
fn subsets(n:usize,k:usize)->Vec<Vec<usize>>{(0u32..1<<n).filter(|m|m.count_ones() as usize==k).map(|m|(0..n).filter(|i|m&(1<<i)!=0).collect()).collect()}
fn chord_default(p:&[i32])->Vec<i32>{let bass=p[0];let pc:BTreeSet<_>=p.iter().map(|p|(p-bass).rem_euclid(12)).collect();if pc.iter().all(|p|*p==0){vec![1,3]}else if pc.iter().all(|p|[0,7].contains(p)){vec![1,2]}else if pc.contains(&10)||pc.contains(&11){vec![1,2,6]}else if pc.contains(&3)&&pc.contains(&7){vec![4,2,3]}else if pc.contains(&4)&&pc.contains(&7){vec![1,2,3]}else if pc.contains(&2)||pc.contains(&5){vec![1,5,3]}else if p.len()==2{vec![1,2]}else{vec![1,2,3]}}
fn shapes(size:usize,barres:bool)->Vec<Vec<i32>>{(1u32..64).filter(|m|m.count_ones() as usize==size).map(|m|(1..=6).filter(|l|m&(1<<(l-1))!=0).collect::<Vec<_>>()).filter(|v|v.iter().map(|l|column(*l)).collect::<BTreeSet<_>>().len()==v.len()||barres&&v.len()==2&&column(v[0])==column(v[1])).collect()}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum ChordFamily{Single,Open,RowNeighbors,RowSkip,Diagonal,OuterDiagonal,Barre,FullRow,MixedTriple,Invalid}
fn chord_family(lanes:&[i32])->ChordFamily{
 if lanes==[0]{return ChordFamily::Open;}if lanes.contains(&0)||lanes.is_empty()||lanes.len()>3{return ChordFamily::Invalid;}if lanes.len()==1{return ChordFamily::Single;}
 let cols=lanes.iter().map(|l|column(*l)).collect::<BTreeSet<_>>();if cols.len()<lanes.len(){return if lanes.len()==2&&cols.len()==1{ChordFamily::Barre}else{ChordFamily::Invalid};}
 let same_row=lanes.iter().all(|l|row(*l)==row(lanes[0]));if lanes.len()==3{return if same_row{ChordFamily::FullRow}else{ChordFamily::MixedTriple};}
 let width=(column(lanes[0])-column(lanes[1])).abs();if same_row{if width==1{ChordFamily::RowNeighbors}else{ChordFamily::RowSkip}}else if width==1{ChordFamily::Diagonal}else{ChordFamily::OuterDiagonal}
}
fn transition(a:&Event,al:&[i32],b:&Event,bl:&[i32],ppq:i64)->i64{
 let movement=center(bl)-center(al);let interval=center(&b.pitches)-center(&a.pitches);let contour=if interval.abs()>0.01&&movement*interval<=0.{1}else{0};
 let row_changes=al.iter().zip(bl).filter(|(a,b)|row(**a)!=row(**b)).count() as i64;
 let jumps=al.iter().zip(bl).map(|(a,b)|(column(*a)-column(*b)).abs().saturating_sub(1) as i64).sum::<i64>();
 let rapid=b.tick-a.tick<ppq/4;let ergonomic=if rapid{row_changes+jumps}else{0};
 if al.len()==1&&bl.len()==1{4*(if a.pitches!=b.pitches&&al==bl{1}else{0})+3*row_changes+2*jumps+2*ergonomic}else{5*contour*(interval.abs().ceil() as i64).clamp(1,3)+4*movement.abs().round() as i64+2*ergonomic}
}
fn map_phrase(e:&mut [Event],options:&Options,ppq:i64)->Result<(),String>{
 let low=pedal(e);let palette:Vec<_>=e.iter().filter(|e|e.pitches.len()==1&&!open(e,low,options)).map(|e|if options.ghl_octave_folding{e.pitches[0].rem_euclid(12)}else{e.pitches[0]}).collect::<BTreeSet<_>>().into_iter().collect();
 if palette.len()>6{return Err(format!("GHL phrase at tick {} has {} distinct melody pitches. Enable GHL octave folding in Song details to fit six buttons.",e[0].tick,palette.len()));}
 let count=palette.len().min(6);let mut best:Option<(i64,usize,Vec<Vec<i32>>)>=None;
 for (id,template) in TEMPLATES.iter().enumerate(){if id==5||template.len()<count{continue;}for subset in subsets(template.len(),count){
 let assignments:Vec<_>=subset.iter().map(|i|template[*i]).collect();
 let choices:Vec<Vec<Vec<i32>>>=e.iter().map(|event|{if open(event,low,options){vec![vec![0]]}else if event.pitches.len()==1{let rank=palette.binary_search(&if options.ghl_octave_folding{event.pitches[0].rem_euclid(12)}else{event.pitches[0]}).unwrap();vec![vec![assignments[rank%assignments.len()]]]}else{let preferred=chord_default(&event.pitches);let mut candidates=shapes(preferred.len(),options.ghl_simple_barres);if preferred.len()==3{candidates.extend(shapes(2,options.ghl_simple_barres));}candidates.sort();candidates}}).collect();
 let mut costs:Vec<Vec<i64>>=vec![];let mut parents:Vec<Vec<usize>>=vec![];
 for (i,event) in e.iter().enumerate(){let mut row_cost=vec![];let mut back=vec![];for lanes in &choices[i]{let mut preferred=if event.pitches.len()>1{chord_default(&event.pitches)}else{lanes.clone()};
 let crowded=(i>0&&event.tick-e[i-1].tick<=ppq/2)||(i+1<e.len()&&e[i+1].tick-event.tick<=ppq/2);
 if options.ghl_simple_barres&&!crowded&&event.pitches.len()==2&&(event.pitches[1]-event.pitches[0]).rem_euclid(12)==0{preferred=vec![1,4];}
 let family=chord_family(lanes);let ergonomic=match family{ChordFamily::OuterDiagonal=>if crowded{6}else{2},ChordFamily::RowSkip=>if crowded{3}else{1},ChordFamily::Barre=>if crowded{8}else if *lanes==preferred{0}else{2},_=>0};
 let local=(if *lanes==preferred{0}else{3})+(if crowded{8}else{2})*(lanes.len().saturating_sub(2)) as i64+ergonomic;
 let mut current=(local,0);if i>0{current=(i64::MAX,0);for (j,prior) in choices[i-1].iter().enumerate(){let mut score=costs[i-1][j]+local+transition(&e[i-1],prior,event,lanes,ppq);if e[i-1].pitches==event.pitches&&prior!=lanes{score+=1000;}if score<current.0{current=(score,j);}}}row_cost.push(current.0);back.push(current.1);}costs.push(row_cost);parents.push(back);}
 let (mut state,cost)=costs.last().unwrap().iter().enumerate().min_by_key(|(i,c)|(**c,*i)).map(|(i,c)|(i,*c)).unwrap();let mut mapping=vec![vec![];e.len()];for i in (0..e.len()).rev(){mapping[i]=choices[i][state].clone();state=parents[i][state];}
 let distinct=mapping.iter().flatten().collect::<BTreeSet<_>>().len();let score=cost+distinct as i64;
 if best.as_ref().is_none_or(|(c,t,m)| (score,id,&mapping)<(*c,*t,m)){best=Some((score,id,mapping));}
 }}let (_,_,mapping)=best.ok_or("GHL phrase has no playable lane assignment")?;for (event,lanes) in e.iter_mut().zip(mapping){event.lanes=lanes;}Ok(())
}
fn expert(e:&mut [Event],score:&Score,o:&Options)->Result<(),String>{let mut families:BTreeMap<Fingerprint,Vec<Vec<i32>>>=BTreeMap::new();for r in phrases(e,score){let key=fingerprint(&e[r.clone()]);if let Some(mapping)=families.get(&key){for (event,lanes) in e[r].iter_mut().zip(mapping){event.lanes=lanes.clone();}}else{map_phrase(&mut e[r.clone()],o,score.ppq)?;
 // Preserve the mapped prefix of a known riff with a different ending.
 let prefix=families.iter().map(|(k,m)|{let n=k.iter().zip(&key).take_while(|(a,b)|a==b).count();(n,m)}).max_by_key(|(n,_)|*n);if let Some((n,m))=prefix{if n>=2{for (event,lanes) in e[r.clone()].iter_mut().zip(m).take(n){event.lanes=lanes.clone();}}}
 families.insert(key,e[r].iter().map(|e|e.lanes.clone()).collect());}}Ok(())}
fn reduce(events:Vec<Event>,ppq:i64,tier:usize)->Vec<Event>{if tier==3{return events;}let grid=[ppq,(ppq/2).max(1),(ppq/4).max(1)][tier];let mut last=None;events.into_iter().filter(|e|{let bucket=e.tick/grid;let keep=last!=Some(bucket);if keep{last=Some(bucket);}keep}).map(|mut e|{if tier==0{e.lanes=vec![if e.lanes==[0]{0}else{column(e.lanes[0])+1}];}else if tier==1&&chord_family(&e.lanes)==ChordFamily::Barre{e.lanes=vec![column(e.lanes[0])+1];}else if e.lanes.len()>2{e.lanes.truncate(2);}e}).collect()}
pub fn generate(score:&Score,o:&Options,track:&Track,tier:usize)->Result<Vec<Gem>,String>{let mut all=group(track);if all.is_empty(){return Ok(vec![]);}expert(&mut all,score,o)?;let events=if tier==3{all}else{let hard=reduce(all,score.ppq,2);if tier==2{hard}else{let medium=reduce(hard,score.ppq,1);if tier==1{medium}else{reduce(medium,score.ppq,0)}}};
 let mut out=vec![];let shift=o.note_shift_ticks(score);let mut prior:Option<&Event>=None;
 for (i,e) in events.iter().enumerate(){let next=events.get(i+1);let raw=e.notes.iter().filter(|n|!n.dead&&!n.palm_mute&&!n.staccato&&!n.tremolo).map(|n|n.length.floor() as i64).min().unwrap_or(0);let length=next.map(|n|raw.min((n.tick-e.tick-(score.ppq/8).max(1)).max(0))).unwrap_or(raw);let length=if length>=score.ppq/2{length}else{0};for lane in &e.lanes{out.push(Gem{tick:e.tick+shift,lane:encode(*lane),length});}
 let repeated=prior.is_some_and(|p|p.pitches==e.pitches||p.lanes==e.lanes);
 let natural=prior.is_some_and(|p|e.lanes.len()==1&&p.lanes!=e.lanes&&e.tick-p.tick<=score.ppq*65/192);
 let picked=e.notes.iter().any(|n|n.picked||n.palm_mute||n.dead||n.staccato||n.tremolo);
 let eligible=prior.is_some_and(|p|p.lanes.len()==1&&e.lanes.len()==1&&p.lanes!=e.lanes)&&!repeated;
 let tap=tier>=2&&e.notes.iter().all(|n|n.tap);
 let desired=tier>=1&&eligible&&!picked&&(e.notes.iter().all(|n|n.hopo)||prior.is_some_and(|p|e.tick-p.tick<=score.ppq/2));
 if tap{out.push(Gem{tick:e.tick+shift,lane:6,length:0});}else if natural!=desired{out.push(Gem{tick:e.tick+shift,lane:5,length:0});}prior=Some(e);
 }out.sort_by_key(|g|(g.tick,g.lane));Ok(out)
}
pub fn barre_pairs(notes:&[Gem])->BTreeSet<(i64,usize)>{let all=notes.iter().filter(|n|preview_lane(n.lane).is_some()).map(|n|(n.tick,n.lane)).collect::<BTreeSet<_>>();let mut out=BTreeSet::new();for (white,black,col) in [(0,3,0),(1,4,1),(2,8,2)]{for &(tick,lane) in &all{if lane==white&&all.contains(&(tick,black)){out.insert((tick,col));}}}out}
fn decode(lane:i32)->Option<i32>{preview_lane(lane).map(|l|if l==6{0}else{l as i32+1})}
fn pattern_name(events:&[(i64,Vec<i32>)],i:usize,ppq:i64)->Option<&'static str>{
 let current=&events[i].1;if chord_family(current)==ChordFamily::Barre{return Some("Simple barre grip");}
 if i+2>=events.len(){return None;}let a=&events[i];let b=&events[i+1];let c=&events[i+2];let ga=b.0-a.0;let gb=c.0-b.0;
 if a.1==b.1&&b.1==c.1{if (ga-gb*2).abs()<=1{return Some("Gallop");}if (gb-ga*2).abs()<=1{return Some("Reverse gallop");}return Some(if current.len()>1{"Chord repetition"}else{"Repeated picking / tremolo"});}
 if current==&vec![0]&&b.1.len()>1&&c.1==[0]{return Some("Open-chord alternation");}
 if current==&c.1&&current.len()==1&&b.1.len()==1{if current==&vec![0]||b.1==[0]{return Some("Open alternation / pedal");}return Some(if column(current[0])==column(b.1[0]){"Same-column row alternation"}else if row(current[0])==row(b.1[0]){"Two-fret trill"}else{"Cross-row zigzag"});}
 if current.len()>1&&b.1.len()>1&&current!=&b.1{let ar=current.iter().all(|l|row(*l)==row(current[0]));let br=b.1.iter().all(|l|row(*l)==row(b.1[0]));return Some(if ar&&br&&row(current[0])!=row(b.1[0]){"Chord row shift"}else if !ar&&!br{"Cross-row chord alternation"}else{"Chord shift"});}
 if current.len()>1&&b.1.len()==1&&current==&c.1{return Some("Chord-to-single turnaround");}
 if i+5<events.len(){let lanes=events[i..i+6].iter().map(|e|e.1.as_slice()).collect::<Vec<_>>();for template in [vec![4,1,5,2,6,3],vec![1,4,2,5,3,6]]{if lanes.iter().zip(&template).all(|(a,b)|*a==[*b]){return Some("Ascending staircase");}if lanes.iter().zip(template.iter().rev()).all(|(a,b)|*a==[*b]){return Some("Descending staircase");}}}
 if current.len()==1&&b.1.len()==1&&c.1.len()==1&&current[0]!=0&&b.1[0]!=0&&c.1[0]!=0&&row(current[0])==row(b.1[0])&&row(b.1[0])==row(c.1[0]){let d1=column(b.1[0])-column(current[0]);let d2=column(c.1[0])-column(b.1[0]);if d1==1&&d2==1||d1== -1&&d2== -1{if (ga-gb).abs()<=1&&(ga*3-ppq).abs()<=3{return Some("Triplet run");}return Some(if d1>0{"Same-row ascending run"}else{"Same-row descending run"});}}
 None
}
pub fn audit(score:&Score,o:&Options,notes:&[Gem],_tier:usize)->Vec<Finding>{
 let timeline=Timeline::new(score,o);let mut groups:BTreeMap<i64,Vec<i32>>=BTreeMap::new();for n in notes{if let Some(lane)=decode(n.lane){groups.entry(n.tick).or_default().push(lane);}}let events=groups.into_iter().collect::<Vec<_>>();let mut out=vec![];let mut seen=BTreeSet::new();
 for (i,(tick,lanes)) in events.iter().enumerate(){let family=chord_family(lanes);let invalid=family==ChordFamily::Invalid||family==ChordFamily::Barre&&!o.ghl_simple_barres;let pattern=if invalid{Some("GHL controller shape")}else if family==ChordFamily::Barre{Some("Simple barre grip — play-test")}else if o.ghl_octave_folding&&i==0{Some("GHL octave folding enabled")}else{pattern_name(&events,i,score.ppq)};
 if let Some(pattern)=pattern{if invalid||family==ChordFamily::Barre||seen.insert(pattern){let (measure,beat)=timeline.position(*tick);out.push(Finding{tick:*tick,measure,beat,bpm:timeline.bpm(*tick),delta_ms:if i>0{(timeline.seconds(*tick as f64)-timeline.seconds(events[i-1].0 as f64))*1000.}else{0.},pattern:pattern.into(),severity:if invalid||family==ChordFamily::Barre{"warning"}else{"info"},suggested_fix:if family==ChordFamily::Barre{"Play-test this square barre note with a six-fret controller in your Clone Hero version."}else{"Compare this six-fret pattern with the audible phrase; avoid forced finger movement."}});}}
 }out
}
#[cfg(test)]mod tests{
 use super::*;
 fn n(t:i64,p:i32)->Note{Note{tick:t as f64,pitch:p,picked:true,..Default::default()}}
 fn s(notes:Vec<Note>)->Score{Score{ppq:480,end:8000.,tracks:vec![Track{index:0,name:"GHL".into(),notes,percussion:false}],tempos:vec![],..Default::default()}}
 fn groups(g:&[Gem])->BTreeMap<i64,Vec<i32>>{let mut out=BTreeMap::new();for n in g{if preview_lane(n.lane).is_some(){out.entry(n.tick).or_insert_with(Vec::new).push(n.lane);}}out}
 #[test]fn all_64_states_are_classified_and_generation_filters_unsafe_shapes(){let mut counts=[0usize;7];for mask in 0u32..64{let lanes=(1..=6).filter(|l|mask&(1<<(l-1))!=0).collect::<Vec<_>>();counts[lanes.len()]+=1;if lanes.len()>=4{assert_eq!(chord_family(&lanes),ChordFamily::Invalid);}}assert_eq!(counts,[1,6,15,20,15,6,1]);assert_eq!(chord_family(&[0]),ChordFamily::Open);assert_eq!(chord_family(&[0,1]),ChordFamily::Invalid);assert_eq!(shapes(2,false).len(),12);assert_eq!(shapes(2,true).len(),15);assert_eq!(shapes(3,true).len(),8);assert!(shapes(4,true).is_empty());assert_eq!(chord_family(&[1,4,2]),ChordFamily::Invalid);}
 #[test]fn crowded_triads_prefer_two_buttons_without_changing_attacks(){let score=s((0..16).flat_map(|i|[60,64,67].map(move|p|Note{length:90.,..n(i*120,p)})).collect());let o=Options::default();let g=generate(&score,&o,&score.tracks[0],3).unwrap();let mapped=groups(&g);assert_eq!(mapped.len(),16);assert!(mapped.values().all(|lanes|lanes.len()==2));let sparse=s([60,64,67].map(|p|Note{length:1440.,..n(0,p)}).to_vec());assert_eq!(groups(&generate(&sparse,&o,&sparse.tracks[0],3).unwrap())[&0].len(),3);}
 #[test]fn optional_simple_barres_are_deterministic_and_lower_tiers_simplify_them(){let score=s((0..4).flat_map(|i|[60,72].map(move|p|Note{length:360.,..n(i*480,p)})).collect());let mut o=Options::default();let normal=generate(&score,&o,&score.tracks[0],3).unwrap();assert!(barre_pairs(&normal).is_empty());o.ghl_simple_barres=true;for tier in 0..4{let g=generate(&score,&o,&score.tracks[0],tier).unwrap();assert_eq!(groups(&g).len(),4);if tier>=2{assert_eq!(barre_pairs(&g).len(),4);assert!(audit(&score,&o,&g,tier).iter().any(|f|f.pattern.contains("barre")));}else{assert!(barre_pairs(&g).is_empty());}assert!(groups(&g).values().all(|l|l.len()<=3));}let g=generate(&score,&o,&score.tracks[0],3).unwrap();assert_eq!(g.iter().map(|g|(g.tick,g.lane,g.length)).collect::<Vec<_>>(),generate(&score,&o,&score.tracks[0],3).unwrap().iter().map(|g|(g.tick,g.lane,g.length)).collect::<Vec<_>>());let fast=s((0..8).flat_map(|i|[60,72].map(move|p|Note{length:60.,..n(i*120,p)})).collect());assert!(barre_pairs(&generate(&fast,&o,&fast.tracks[0],3).unwrap()).is_empty());}
 #[test]fn reference_patterns_are_recognized_without_converting_them_to_modifiers(){for (ticks,lanes,name) in [(vec![0,240,360],vec![vec![4],vec![4],vec![4]],"Gallop"),(vec![0,120,360],vec![vec![4],vec![4],vec![4]],"Reverse gallop"),(vec![0,160,320],vec![vec![4],vec![5],vec![6]],"Triplet run"),(vec![0,120,240],vec![vec![4],vec![1],vec![4]],"Same-column row alternation"),(vec![0,120,240],vec![vec![4],vec![2],vec![4]],"Cross-row zigzag"),(vec![0,120,240],vec![vec![0],vec![4,5],vec![0]],"Open-chord alternation")]{let e=ticks.into_iter().zip(lanes).collect::<Vec<_>>();assert_eq!(pattern_name(&e,0,480),Some(name));}for (lanes,name) in [(vec![4,1,5,2,6,3],"Ascending staircase"),(vec![3,6,2,5,1,4],"Descending staircase")]{let e=lanes.iter().enumerate().map(|(i,l)|(i as i64*120,vec![*l])).collect::<Vec<_>>();assert_eq!(pattern_name(&e,0,480),Some(name));assert!(TEMPLATES.iter().any(|t|*t==[4,1,5,2,6,3]));}}
 #[test]fn encoding_and_six_note_contour(){assert_eq!((1..=6).map(encode).collect::<Vec<_>>(),vec![0,1,2,3,4,8]);let score=s((0..6).map(|i|n(i*120,60+i as i32)).collect());let o=Options::default();let a=generate(&score,&o,&score.tracks[0],3).unwrap();assert_eq!(a.iter().filter(|n|preview_lane(n.lane).is_some()).map(|n|n.lane).collect::<Vec<_>>(),vec![0,3,1,4,2,8]);assert_eq!(serde_json::to_string(&a.iter().map(|g|(g.tick,g.lane,g.length)).collect::<Vec<_>>()).unwrap(),serde_json::to_string(&generate(&score,&o,&score.tracks[0],3).unwrap().iter().map(|g|(g.tick,g.lane,g.length)).collect::<Vec<_>>()).unwrap());}
 #[test]fn riff_transposition_and_pedal_policy(){let mut notes=vec![];for (start,transpose) in [(0,0),(3000,12)]{for (i,p) in [40,40,43,40,45,40,43,45].iter().enumerate(){notes.push(n(start+i as i64*120,p+transpose));}}let score=s(notes);let o=Options::default();let mapped=groups(&generate(&score,&o,&score.tracks[0],3).unwrap());for i in 0..8{assert_eq!(mapped[&(i*120)],mapped[&(3000+i*120)]);}assert_eq!(mapped[&0],vec![7]);let mut o=o;o.ghl_open_pedals=false;assert!(!generate(&score,&o,&score.tracks[0],3).unwrap().iter().any(|n|n.lane==7));let strings=s(vec![Note{fret:Some(0),..n(0,40)},n(480,42)]);assert!(!generate(&strings,&o,&strings.tracks[0],3).unwrap().iter().any(|n|n.lane==7));}
 #[test]fn chord_ergonomics_and_articulations(){let score=s([40,43,45,40].iter().enumerate().flat_map(|(i,p)|[Note{length:240.,..n(i as i64*480,*p)},Note{length:240.,..n(i as i64*480,p+7)}]).collect());let o=Options::default();let result=generate(&score,&o,&score.tracks[0],3).unwrap();assert!(audit(&score,&o,&result,3).iter().all(|f|f.severity!="warning"));let mapped=groups(&result);assert_eq!(mapped[&0],mapped[&1440]);assert_ne!(mapped[&0],mapped[&480]);let score=s(vec![n(0,60),Note{hopo:true,picked:false,..n(120,62)},n(240,64),Note{tap:true,picked:false,..n(360,65)}]);let g=generate(&score,&o,&score.tracks[0],3).unwrap();assert!(g.iter().any(|n|n.tick==240&&n.lane==5));assert!(g.iter().any(|n|n.tick==360&&n.lane==6));}
 #[test]fn sustain_reductions_and_overflow(){let score=s(vec![Note{length:1440.,..n(0,60)},n(120,62),n(480,64)]);let o=Options::default();let easy=generate(&score,&o,&score.tracks[0],0).unwrap();assert_eq!(easy.iter().find(|g|g.tick==0&&g.lane<5).unwrap().length,420);let g=generate(&score,&o,&score.tracks[0],3).unwrap();assert_eq!(g.iter().find(|g|g.tick==0&&g.lane<5).unwrap().length,0);let score=s([60,61,62,63,64,65,72].iter().enumerate().map(|(i,p)|n(i as i64*120,*p)).collect());assert!(generate(&score,&o,&score.tracks[0],3).is_err());let mut o=o;o.ghl_octave_folding=true;assert!(generate(&score,&o,&score.tracks[0],3).is_ok());}
}
