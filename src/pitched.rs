//! Rulebook v1.0 authoring policy shared by Guitar, Rhythm and Bass.
//! Exact attacks are retained on Expert. Review targets are not parser limits.
use crate::{engine::{Gem,Note,Options,Score,Track},timing::Timeline};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Clone)]
struct Event {tick:i64,notes:Vec<Note>,pitches:Vec<i32>,lanes:Vec<i32>}
fn attacks(track:&Track)->Vec<Event>{
    let mut groups:BTreeMap<i64,Vec<Note>>=BTreeMap::new();
    for note in &track.notes{groups.entry(note.tick.round() as i64).or_default().push(note.clone());}
    groups.into_iter().map(|(tick,notes)|{let pitches=notes.iter().map(|n|n.pitch).collect::<BTreeSet<_>>().into_iter().collect();Event{tick,notes,pitches,lanes:Vec::new()}}).collect()
}
fn phrase_ranges(events:&[Event],score:&Score)->Vec<std::ops::Range<usize>>{
    let mut ranges=Vec::new();let mut first=0;
    for i in 1..events.len(){
        let prior=&events[i-1];let release=prior.notes.iter().map(|n|prior.tick+n.length.round() as i64).max().unwrap_or(prior.tick);
        let rest=events[i].tick-release.max(prior.tick)>score.ppq;
        let section=score.sections.iter().any(|s|s.tick>prior.tick as f64&&s.tick<=events[i].tick as f64);
        if rest||section||events[i].tick-events[first].tick>=score.ppq*32||i-first>=128{ranges.push(first..i);first=i;}
    }
    if first<events.len(){ranges.push(first..events.len());}ranges
}
fn voicing(pitches:&[i32])->Vec<i32>{
    if pitches.len()<=2{return pitches.to_vec();}
    let root=pitches[0];let power=pitches.iter().all(|p|[0,7].contains(&(p-root).rem_euclid(12)));
    if power{return vec![root,*pitches.iter().find(|p|(**p-root).rem_euclid(12)==7).unwrap_or(&pitches[pitches.len()-1])];}
    vec![root,pitches[pitches.len()/2],pitches[pitches.len()-1]]
}
fn combinations(count:usize)->Vec<Vec<i32>>{
    (1_u32..32).filter(|mask|mask.count_ones() as usize==count).map(|mask|(0..5).filter(|lane|mask&(1<<lane)!=0).collect()).collect()
}
fn center(lanes:&[i32])->f64{lanes.iter().sum::<i32>() as f64/lanes.len() as f64}
fn pitch_center(pitches:&[i32])->f64{pitches.iter().sum::<i32>() as f64/pitches.len() as f64}
fn map_phrase(events:&mut [Event]){
    let palette:Vec<_>=events.iter().flat_map(|e|e.pitches.iter().copied()).collect::<BTreeSet<_>>().into_iter().collect();
    if palette.len()<=5 {
        for event in events{event.lanes=voicing(&event.pitches).into_iter().map(|p|{let rank=palette.binary_search(&p).unwrap();if palette.len()==1{0}else{(rank*4/(palette.len()-1)) as i32}}).collect();event.lanes.sort_unstable();event.lanes.dedup();}
        return;
    }
    // Choose fret positions for the entire phrase, considering the next attacks.
    // Larger pitch runs can reset position rather than collapse adjacent pitches.
    let choices:Vec<_>=events.iter().map(|e|combinations(voicing(&e.pitches).len())).collect();
    let mut costs:Vec<Vec<f64>>=Vec::new();let mut parents:Vec<Vec<usize>>=Vec::new();
    let low=palette[0] as f64;let range=(palette[palette.len()-1] as f64-low).max(1.);
    for (i,event) in events.iter().enumerate(){
        let mut row=Vec::new();let mut back=Vec::new();
        for lanes in &choices[i]{
            let desired=(pitch_center(&event.pitches)-low)/range*4.;let local=(center(lanes)-desired).powi(2)*0.8;
            let mut best=(local,0);
            if i>0{
                best=(f64::INFINITY,0);
                for (j,previous) in choices[i-1].iter().enumerate(){
                    let source=pitch_center(&event.pitches)-pitch_center(&events[i-1].pitches);let movement=center(lanes)-center(previous);
                    let mut transition=movement.abs()*0.3;
                    if event.pitches==events[i-1].pitches&&lanes!=previous{transition+=1000.;}
                    if source!=0.&&movement*source<=0.{transition+=if movement==0.{25.}else{16.};}
                    if source==0.&&movement!=0.{transition+=5.;}
                    let candidate=costs[i-1][j]+local+transition;
                    if candidate<best.0{best=(candidate,j);}
                }
            }
            row.push(best.0);back.push(best.1);
        }
        costs.push(row);parents.push(back);
    }
    let mut state=costs.last().unwrap().iter().enumerate().min_by(|a,b|a.1.total_cmp(b.1)).unwrap().0;
    for i in (0..events.len()).rev(){events[i].lanes=choices[i][state].clone();state=parents[i][state];}
}
fn expert(events:&mut [Event],score:&Score){
    // Identical phrases share a mapping, including their relative rhythm.
    let mut motifs:BTreeMap<Vec<(i64,Vec<i32>)>,Vec<Vec<i32>>>=BTreeMap::new();
    for range in phrase_ranges(events,score){let start=events[range.start].tick;let key=events[range.clone()].iter().map(|e|(e.tick-start,e.pitches.clone())).collect();
        if let Some(lanes)=motifs.get(&key){for (e,l) in events[range].iter_mut().zip(lanes){e.lanes=l.clone();}}
        else{map_phrase(&mut events[range.clone()]);motifs.insert(key,events[range].iter().map(|e|e.lanes.clone()).collect());}
    }
}
fn importance(events:&[Event],i:usize,ppq:i64)->i32{
    let e=&events[i];let mut score=if e.tick%(ppq*4)==0{12}else if e.tick%ppq==0{8}else if e.tick%(ppq/2).max(1)==0{4}else{1};
    if i==0||i+1==events.len(){score+=8;}
    if e.notes.iter().any(|n|n.accent||n.velocity>=115){score+=5;}
    if e.pitches.len()>1{score+=3;}
    if i>0&&i+1<events.len(){let a=pitch_center(&e.pitches)-pitch_center(&events[i-1].pitches);let b=pitch_center(&events[i+1].pitches)-pitch_center(&e.pitches);if a*b<0.{score+=4;}}
    score
}
fn arrange(events:Vec<Event>,score:&Score,options:&Options,tier:usize)->Vec<Event>{
    if tier==3{return events;}
    let timeline=Timeline::new(score,options);let shift=options.note_shift_ticks(score);
    if tier==2 {
        // Keep solos, gallops and short bursts; thin only sustained punishing
        // repeated picking / chord streams, not everything under 90 ms.
        let mut keep=vec![true;events.len()];let mut first=0;
        while first<events.len(){let mut end=first+1;
            while end<events.len(){let a=&events[end-1];let b=&events[end];let gap=timeline.seconds((b.tick+shift) as f64)-timeline.seconds((a.tick+shift) as f64);if gap>=0.09||!(a.pitches==b.pitches||a.pitches.len()>1&&b.pitches.len()>1)||a.notes.iter().chain(&b.notes).any(|n|n.hopo||n.tap){break;}end+=1;}
            if end-first>=8{for i in first+1..end-1{keep[i]=(i-first)%2==0;}}
            first=end;
        }
        return events.into_iter().zip(keep).filter_map(|(e,keep)|keep.then_some(e)).collect();
    }
    let grid=if tier==0{score.ppq}else{(score.ppq/2).max(1)};
    let mut selected=Vec::new();let mut bucket=None;let mut winner=0;
    for i in 0..events.len(){let next=events[i].tick/grid;
        if bucket!=Some(next){if bucket.is_some(){selected.push(events[winner].clone());}bucket=Some(next);winner=i;}
        else if importance(&events,i,score.ppq)>importance(&events,winner,score.ppq){winner=i;}
    }
    if bucket.is_some(){selected.push(events[winner].clone());}selected
}
fn reduced_lanes(event:&Event,tier:usize)->Vec<i32>{
    if tier==3{return event.lanes.clone();}
    if tier==0{return vec![(event.lanes.last().copied().unwrap_or(0) as f64/2.).round() as i32];}
    let mut lanes=event.lanes.clone();
    if lanes.len()>2{lanes=vec![lanes[0],lanes[lanes.len()-1]];}
    if tier==1{for lane in &mut lanes{*lane=(*lane as f64*0.75).round() as i32;}}
    lanes.sort_unstable();lanes.dedup();lanes
}
pub fn clearance_ticks(score:&Score,options:&Options,tier:usize,end:i64,awkward:bool)->i64{
    let beat=[0.25,0.125,0.125,0.0625][tier];let ms=if awkward{[150.,100.,75.,60.][tier]}else{[100.,70.,50.,30.][tier]};
    let timeline=Timeline::new(score,options);let limit=timeline.tick((timeline.seconds(end as f64)-ms/1000.).max(0.));
    ((score.ppq as f64*beat).ceil() as i64).max((end as f64-limit).ceil() as i64)
}
pub fn generate(score:&Score,options:&Options,track:&Track,role:usize,tier:usize)->Vec<Gem>{
    let mut all=attacks(track);expert(&mut all,score);
    // Hierarchical reductions keep lower attacks a subset of the upper tiers.
    let hard=if tier<3{arrange(all.clone(),score,options,2)}else{all.clone()};
    let medium=if tier<2{arrange(hard.clone(),score,options,1)}else{hard.clone()};
    let events=match tier{0=>arrange(medium,score,options,0),1=>medium,2=>hard,_=>all};
    let shift=options.note_shift_ticks(score);let mut result=Vec::new();let mut previous:Option<(i64,Vec<i32>,Vec<i32>)>=None;
    for (i,event) in events.iter().enumerate(){
        let mut lanes=reduced_lanes(event,tier);
        // Open strings describe tuning, not purple gameplay. Only explicit
        // source gameplay annotations may enable this optional Expert mechanic.
        if role==2&&tier==3&&options.open_bass&&event.notes.iter().all(|n|n.chart_open)&&event.pitches.len()==1{lanes=vec![7];}
        let next=events.get(i+1);let next_lanes=next.map(|e|reduced_lanes(e,tier));
        let awkward=next_lanes.as_ref().is_some_and(|l|l.len()>1&&l!=&lanes||((center(l)-center(&lanes)).abs()>2.));
        let tick=event.tick+shift;
        let permitted=next.map(|n|(n.tick-event.tick-clearance_ticks(score,options,tier,n.tick+shift,awkward)).max(0));
        for &lane in &lanes{
            // Keep individual chord releases. A reduced lane uses the matching
            // voice's duration; removed attacks do not cut retained sustains.
            let source_lane=if tier==0{event.lanes.last().copied().unwrap_or(0)}else if tier==1{event.lanes.iter().copied().find(|l|(*l as f64*0.75).round() as i32==lane).unwrap_or(lane)}else{lane};
            let voice=event.lanes.iter().position(|l|*l==source_lane).unwrap_or(0);
            let pitches=voicing(&event.pitches);let pitch=pitches.get(voice).copied().unwrap_or(event.pitches[0]);
            let raw=event.notes.iter().filter(|n|n.pitch==pitch&&!n.dead&&!n.palm_mute&&!n.staccato).map(|n|n.length.floor() as i64).max().unwrap_or(0);
            let length=permitted.map(|max|raw.min(max)).unwrap_or(raw);let length=if length>=score.ppq/2{length}else{0};
            result.push(Gem{tick,lane,length});
        }
        let repeated=previous.as_ref().is_some_and(|p|p.2==event.pitches);
        let eligible=previous.as_ref().is_some_and(|p|p.0<tick&&p.1!=lanes)&&lanes.len()==1&&!repeated;
        let tap=tier==3&&!repeated&&event.notes.iter().all(|n|n.tap);
        let hopo=tier>=1&&eligible&&event.notes.iter().all(|n|n.hopo||tier==2&&n.tap)&&!event.notes.iter().any(|n|n.dead||n.palm_mute||n.staccato);
        // .chart N 5 toggles the automatic type; it is not force-HOPO.
        let natural=previous.as_ref().is_some_and(|p|tick-p.0<=score.ppq*65/192&&p.1!=lanes)&&lanes.len()==1;
        if tap{result.push(Gem{tick,lane:6,length:0});}else if hopo!=natural{result.push(Gem{tick,lane:5,length:0});}
        previous=Some((tick,lanes,event.pitches.clone()));
    }
    result.sort_by_key(|g|(g.tick,g.lane));result
}
#[derive(Clone,serde::Serialize)]
pub struct Finding {pub tick:i64,pub measure:i64,pub beat:f64,pub bpm:f64,pub delta_ms:f64,pub pattern:String,pub severity:&'static str,pub suggested_fix:&'static str}
pub fn audit(score:&Score,options:&Options,notes:&[Gem],tier:usize)->Vec<Finding>{
    let mut groups:BTreeMap<i64,Vec<&Gem>>=BTreeMap::new();for note in notes.iter().filter(|n|(0..=4).contains(&n.lane)||n.lane==7){groups.entry(note.tick).or_default().push(note);}
    let events:Vec<_>=groups.into_iter().collect();let timeline=Timeline::new(score,options);let mut findings=Vec::new();
    let mut flag=|tick:i64,delta:f64,pattern:String,severity,fix|{let (measure,beat)=timeline.position(tick);findings.push(Finding{tick,measure,beat,bpm:timeline.bpm(tick),delta_ms:(delta*100.).round()/100.,pattern,severity,suggested_fix:fix});};
    let shape=|notes:&[&Gem]|notes.iter().map(|n|["G","R","Y","B","O","?","?","P"][n.lane as usize]).collect::<Vec<_>>().join("+");
    for pair in events.windows(2){let (tick,a)=&pair[0];let (next,b)=&pair[1];let gap=(timeline.seconds(*next as f64)-timeline.seconds(*tick as f64))*1000.;let pattern=format!("{} → {}",shape(a),shape(b));
        if gap<[250.,150.,90.,60.][tier]{flag(*next,gap,format!("Dense attacks: {pattern}"),if tier==3{"info"}else{"warning"},"Compare to the recording and playtest; keep justified bursts and syncopation.");}
        if gap<10.{flag(*next,gap,format!("Possible staggered chord: {pattern}"),"warning","Verify whether these are one simultaneous attack or distinct grace/strummed attacks; do not snap without listening.");}
        let wide=(center(&a.iter().map(|n|n.lane).collect::<Vec<_>>())-center(&b.iter().map(|n|n.lane).collect::<Vec<_>>())).abs()>2.;
        if gap<250.&&(wide||a.len()>1&&b.len()>1&&shape(a)!=shape(b)||a.len()>=3&&b.len()>=3||a.iter().any(|n|n.lane==7)!=b.iter().any(|n|n.lane==7)){flag(*next,gap,format!("Release/regrip: {pattern}"),"warning","Review the jump or chord change at song speed; simplify the hand movement if needed.");}
        for sustain in a.iter().filter(|n|n.length>0){let end=sustain.tick+sustain.length;let clearance=(timeline.seconds(*next as f64)-timeline.seconds(end as f64))*1000.;let required=clearance_ticks(score,options,tier,*next,wide||b.len()>1&&shape(a)!=shape(b));if end>*next-required{flag(*tick,clearance,format!("Sustain clearance: {pattern}"),"warning","Shorten this tail or confirm an intentionally supported overlapping sustain.");}}
    }
    findings
}
#[cfg(test)] mod tests{
    use super::*;
    fn score(notes:Vec<Note>)->Score{Score{ppq:480,end:10000.,tracks:vec![Track{index:0,name:"Guitar".into(),percussion:false,notes}],..Default::default()}}
    fn note(tick:i64,pitch:i32)->Note{Note{tick:tick as f64,pitch,velocity:100,..Default::default()}}
    fn options()->Options{Options{tracks:[None,Some(0),Some(0),Some(0),None,None,None],..Default::default()}}
    fn playable(notes:&[Gem])->BTreeMap<i64,Vec<i32>>{let mut result:BTreeMap<i64,Vec<i32>>=BTreeMap::new();for n in notes.iter().filter(|n|(0..=4).contains(&n.lane)||n.lane==7){result.entry(n.tick).or_default().push(n.lane);}result}
    #[test]fn expert_retains_dense_attacks_and_open_strings_stay_colored(){let s=score((0..20).map(|i|Note{fret:Some(0),hopo:true,tap:true,..note(i*30,40)}).collect());let o=options();for role in 1..4{let notes=generate(&s,&o,&s.tracks[0],role,3);assert_eq!(playable(&notes).len(),20);assert!(!notes.iter().any(|n|n.lane==7));assert!(!notes.iter().any(|n|n.tick>0&&(n.lane==5||n.lane==6)));assert!(audit(&s,&o,&notes,3).iter().any(|f|f.severity=="info"));}}
    #[test]fn force_modifier_toggles_natural_hopo_in_both_directions(){let s=score(vec![note(0,60),Note{hopo:true,..note(120,62)},note(240,64),Note{hopo:true,..note(720,65)},Note{hopo:true,..note(840,65)}]);let o=options();let notes=generate(&s,&o,&s.tracks[0],1,3);assert_eq!(notes.iter().filter(|n|n.lane==5).map(|n|n.tick).collect::<Vec<_>>(),vec![240,720]);assert!(!notes.iter().any(|n|n.lane==6));}
    #[test]fn hierarchy_obeys_lane_chord_and_note_type_limits(){let s=score((0..48).flat_map(|i|(0..6).map(move|j|Note{tap:true,chart_open:true,..note(i*120+40,40+j*2+(i%3) as i32)})).collect());let o=options();for role in 1..4{let tiers:Vec<_>=(0..4).map(|t|generate(&s,&o,&s.tracks[0],role,t)).collect();for t in 0..4{let groups=playable(&tiers[t]);assert!(groups.values().all(|lanes|lanes.len()<=[1,2,2,3][t]));assert!(groups.values().flatten().all(|lane|*lane<[3,4,5,5][t]));assert!(groups.keys().all(|tick|s.tracks[0].notes.iter().any(|n|n.tick==*tick as f64)));if t<3{assert!(!tiers[t].iter().any(|n|n.lane==6||n.lane==7));assert!(groups.keys().all(|tick|playable(&tiers[t+1]).contains_key(tick)));}}}}
    #[test]fn hard_keeps_short_fast_bursts_but_thins_sustained_strumming(){let o=options();let burst=score((0..4).map(|i|note(i*60,60)).collect());assert_eq!(playable(&generate(&burst,&o,&burst.tracks[0],1,2)).len(),4);let stream=score((0..16).map(|i|note(i*60,60)).collect());let hard=generate(&stream,&o,&stream.tracks[0],1,2);assert!(playable(&hard).len()<16);assert_eq!(playable(&generate(&stream,&o,&stream.tracks[0],1,3)).len(),16);}
    #[test]fn repeated_phrases_chords_and_contours_stay_consistent(){let pitches=[60,62,64,65,67,65,64,62,60];let mut notes=Vec::new();for start in [0,6000]{for (i,p) in pitches.iter().enumerate(){notes.push(note(start+i as i64*120,*p));}}let s=score(notes);let o=options();let events=playable(&generate(&s,&o,&s.tracks[0],1,3));for i in 0..pitches.len(){assert_eq!(events[&(i as i64*120)],events[&(6000+i as i64*120)]);}for i in 0..4{assert!(events[&(i*120)][0]<events[&((i+1)*120)][0]);}let chord=score(vec![note(0,40),note(0,47),note(0,52),note(480,40),note(480,47),note(480,52)]);let mapped=playable(&generate(&chord,&o,&chord.tracks[0],3,3));assert_eq!(mapped[&0].len(),2);assert_eq!(mapped[&0],mapped[&480]);}
    #[test]fn long_pitch_runs_reset_positions_without_losing_contour_or_attacks(){let pitches=[60,61,62,63,64,65,66,67,68,68,68];let mut notes=Vec::new();for start in [0,6000]{for (i,p) in pitches.iter().enumerate(){notes.push(note(start+i as i64*120,*p));}}let s=score(notes);let o=options();let result=generate(&s,&o,&s.tracks[0],1,3);let events=playable(&result);assert_eq!(events.len(),22);let lanes:Vec<_>=(0..9).map(|i|events[&(i*120)][0]).collect();assert!(lanes.windows(2).all(|p|p[0]!=p[1]));assert!(lanes.windows(2).filter(|p|p[0]>p[1]).count()<=2);for i in 0..11{assert_eq!(events[&(i*120)],events[&(6000+i*120)]);}assert_eq!(events[&960],events[&1080]);assert_eq!(events[&1080],events[&1200]);}
    #[test]fn sustained_notes_use_retained_next_attack_and_actual_tempo_clearance(){let mut s=score(vec![Note{length:1440.,..note(0,60)},note(120,62),note(480,64)]);let o=options();let easy=generate(&s,&o,&s.tracks[0],1,0);let first=easy.iter().find(|n|n.tick==0&&n.lane<5).unwrap();assert_eq!(first.length,360);assert!(audit(&s,&o,&easy,0).iter().all(|f|!f.pattern.starts_with("Sustain clearance")));s.tempos=vec![crate::engine::Tempo{tick:0.,bpm:120.},crate::engine::Tempo{tick:440.,bpm:300.}];let fast=generate(&s,&o,&s.tracks[0],1,0);let end=fast.iter().find(|n|n.tick==0&&n.lane<5).unwrap().length;let t=Timeline::new(&s,&o);assert!(t.seconds(480.)-t.seconds(end as f64)>=0.1-1e-9);assert!(end>=240);}
    #[test]fn independent_chord_releases_and_explicit_expert_open_are_preserved(){let s=score(vec![Note{length:480.,..note(0,60)},Note{length:960.,..note(0,64)},note(1920,65)]);let o=options();let notes=generate(&s,&o,&s.tracks[0],1,3);assert_eq!(notes.iter().filter(|n|n.tick==0&&n.lane<5).map(|n|n.length).collect::<Vec<_>>(),vec![480,960]);let open=score(vec![Note{chart_open:true,..note(0,40)},note(480,42)]);for tier in 0..4{let notes=generate(&open,&o,&open.tracks[0],2,tier);assert_eq!(notes.iter().any(|n|n.lane==7),tier==3);}}
}
