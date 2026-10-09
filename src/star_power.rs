//! Musical Star Power placement. Four complete measures between phrases is a hard limit.
use crate::{engine::{self,Gem,Options,Score},timing::Timeline};
use std::collections::BTreeMap;
#[derive(Clone,Debug,serde::Serialize)]
pub struct Phrase{pub start:i64,pub length:i64,pub measures:f64,pub hits:usize}
#[derive(Clone,Default,serde::Serialize)]
pub struct Plan{pub phrases:Vec<Phrase>,pub warnings:Vec<String>}
struct Grid{starts:Vec<i64>}
impl Grid{
 fn new(score:&Score,last:i64)->Self{let mut signatures:BTreeMap<i64,(i64,i64)>=BTreeMap::from([(0,(4,4))]);for s in &score.signatures{if s.tick>=0.&&s.numerator>0&&s.denominator>0{signatures.insert(s.tick.round() as i64,(s.numerator as i64,s.denominator as i64));}}
 let mut starts=vec![0];let limit=last.max(score.end.ceil() as i64)+score.ppq*32;
 while *starts.last().unwrap()<=limit{let tick=*starts.last().unwrap();let (_, &(n,d))=signatures.range(..=tick).next_back().unwrap();let next=(tick+score.ppq*n*4/d).max(tick+1);let change=signatures.range(tick+1..next).next().map(|(t,_)|*t);starts.push(change.unwrap_or(next));}Self{starts}}
 fn measure(&self,tick:i64)->f64{let i=self.starts.partition_point(|t|*t<=tick).saturating_sub(1).min(self.starts.len()-2);i as f64+(tick-self.starts[i]) as f64/(self.starts[i+1]-self.starts[i]) as f64}
 fn tick(&self,measure:f64)->i64{let measure=measure.max(0.);let i=(measure.floor() as usize).min(self.starts.len()-2);(self.starts[i] as f64+(measure-i as f64)*(self.starts[i+1]-self.starts[i]) as f64).round() as i64}
}
#[derive(Clone)]struct Attack{tick:i64,hits:usize,strength:f64}
pub fn plan(score:&Score,o:&Options,notes:&[Gem],role:usize)->Plan{
 if !o.star_power{return Plan::default();}let shift=o.note_shift_ticks(score);let timeline=Timeline::new(score,o);let mut grouped:BTreeMap<i64,usize>=BTreeMap::new();for n in notes.iter().filter(|n|engine::playable(n.lane)){*grouped.entry(n.tick-shift).or_default()+=1;}
 if grouped.is_empty(){return Plan::default();}let last=*grouped.last_key_value().unwrap().0;let grid=Grid::new(score,last);
 let source=o.tracks.get(role).and_then(|i|*i).and_then(|i|score.tracks.iter().find(|t|t.index==i));let mut emphasis:BTreeMap<i64,f64>=BTreeMap::new();if let Some(track)=source{for n in &track.notes{let value=if n.accent{8.}else{0.}+((n.velocity-100).max(0) as f64/8.);let v=emphasis.entry(n.tick.round() as i64).or_default();*v=v.max(value);}}
 let attacks:Vec<_>=grouped.into_iter().map(|(tick,hits)|{let m=grid.measure(tick);let downbeat=(m-m.round()).abs()<1e-6;let quarter=tick%score.ppq==0;Attack{tick,hits,strength:if downbeat{12.}else if quarter{3.}else{0.}+hits.min(3) as f64*2.+emphasis.get(&tick).copied().unwrap_or(0.)}}).collect();
 let mut boundaries:Vec<f64>=score.sections.iter().map(|s|grid.measure(s.tick.round() as i64)).collect();boundaries.extend(score.signatures.iter().skip(1).map(|s|grid.measure(s.tick.round() as i64)));boundaries.extend(score.tempos.iter().skip(1).filter(|_|!o.constant_bpm).map(|s|grid.measure(s.tick.round() as i64)));
 // Rhythm/density transitions are useful when the GP score has no section markers.
 let mut per_bar:BTreeMap<i64,usize>=BTreeMap::new();for a in &attacks{*per_bar.entry(grid.measure(a.tick).floor() as i64).or_default()+=a.hits;}for (&bar,&hits) in &per_bar{if bar>0{let previous=per_bar.get(&(bar-1)).copied().unwrap_or(0);if hits>=previous.max(1)*2||previous>=hits.max(1)*2{boundaries.push(bar as f64);}}}
 let seconds=|tick:i64|timeline.seconds((tick+shift).max(0) as f64);
 let mut result=Plan::default();let mut previous_end:Option<i64>=None;let mut next_index=0;
 while next_index<attacks.len(){let earliest=previous_end.map(|end|grid.tick(grid.measure(end)+4.)+1).unwrap_or(attacks[0].tick);let first=attacks.partition_point(|a|a.tick<earliest).max(next_index);if first>=attacks.len(){break;}
 let deadline=previous_end.map(|end|timeline.tick(seconds(end)+30.).floor() as i64-shift).unwrap_or_else(||timeline.tick(seconds(attacks[0].tick)+15.).floor() as i64-shift);
 let mut stop=attacks.partition_point(|a|a.tick<=deadline).max(first+1).min(attacks.len());
 // Do not create a terminal phrase with no usable notes simply to satisfy a timer.
 if stop<=first{stop=first+1;}
 let chosen=(first..stop).min_by(|&a,&b|{let cost=|i:usize|{let note=&attacks[i];let m=grid.measure(note.tick);let proximity=boundaries.iter().map(|boundary|(m-boundary).abs().min((m+2.-boundary).abs())).fold(f64::INFINITY,f64::min);let anchor_bonus=if proximity<=0.5{5.*(1.-proximity)}else{0.};
 let distance=previous_end.map(|end|0.6*(m-grid.measure(end)-8.).abs()+0.5*(seconds(note.tick)-seconds(end)-15.).abs()).unwrap_or(0.8*(seconds(note.tick)-seconds(attacks[0].tick)));
 distance-anchor_bonus-note.strength*0.04};cost(a).total_cmp(&cost(b)).then(a.cmp(&b))}).unwrap();let start=&attacks[chosen];let m=grid.measure(start.tick);
 let nearby:Vec<_>=attacks.iter().skip(chosen).take_while(|a|grid.measure(a.tick)<m+2.).collect();let density=nearby.iter().map(|a|a.hits).sum::<usize>() as f64/2.;let duration=if density>=16.{1.}else if density<4.{4.}else{2.};
 let min_end=grid.tick(m+1.);let max_end=grid.tick(m+4.);let target=m+duration;
 let end_hit=attacks.iter().enumerate().skip(chosen).take_while(|(_,a)|a.tick<=max_end).filter(|(_,a)|a.tick>=min_end).min_by(|(ia,a),(ib,b)|{let cost=|a:&Attack|(grid.measure(a.tick)-target).abs()*20.-a.strength;cost(a).total_cmp(&cost(b)).then(ia.cmp(ib))}).map(|(i,_)|i).unwrap_or_else(||attacks.partition_point(|a|a.tick<=max_end).saturating_sub(1).max(chosen));
 let end=(attacks[end_hit].tick+1).min((score.end.ceil() as i64+1).max(start.tick+1));let measures=grid.measure(end)-m;let hits=attacks[chosen..=end_hit].iter().map(|a|a.hits).sum();
 if measures<1.-1e-6{result.warnings.push(format!("Phrase at {:.2}s is shorter than one measure because the remaining playable passage is short.",seconds(start.tick)));}
 result.phrases.push(Phrase{start:start.tick+shift,length:(end-start.tick).max(1),measures,hits});previous_end=Some(end);next_index=end_hit+1;
 }
 // Include the first/last playable passage in the 30-second coverage audit; silence cannot grant SP.
 let mut edge=attacks[0].tick;for phrase in &result.phrases{let start=phrase.start-shift;let gap=seconds(start)-seconds(edge);if gap>30.+1e-6{result.warnings.push(format!("{gap:.1}s without a Star Power phrase. Four-measure spacing or a lack of playable notes prevents the 30-second target."));}edge=start+phrase.length;}
 let tail=seconds(last)-seconds(edge);if tail>30.+1e-6{result.warnings.push(format!("Final playable passage goes {tail:.1}s without Star Power; no further phrase fits the four-measure minimum gap."));}result
}
#[cfg(test)]mod tests{
 use super::*;use crate::engine::{Track,Note,Signature,Tempo,Section};
 fn fixture(bpm:f64,bars:i64,hits_per_bar:i64)->(Score,Options,Vec<Gem>){let ppq=480;let notes:Vec<_>=(0..bars*hits_per_bar).map(|i|Gem{tick:i*1920/hits_per_bar,lane:1,length:0}).collect();let score=Score{ppq,end:(bars*1920) as f64,tracks:vec![Track{index:0,name:"Drums".into(),percussion:true,notes:notes.iter().map(|g|Note{tick:g.tick as f64,pitch:38,velocity:100,..Default::default()}).collect()}],tempos:vec![Tempo{tick:0.,bpm}],..Default::default()};let o=Options{bpm,tracks:[Some(0),None,None,None,None,None,None],..Default::default()};(score,o,notes)}
 #[test]fn standard_two_bar_phrases_minimum_spacing_and_determinism(){let (s,o,n)=fixture(120.,64,8);let p=plan(&s,&o,&n,0);assert!(p.phrases.len()>=5);assert!(p.warnings.is_empty());assert_eq!(serde_json::to_string(&p).unwrap(),serde_json::to_string(&plan(&s,&o,&n,0)).unwrap());for phrase in &p.phrases{assert!(phrase.measures>=1.&&phrase.measures<=4.001);assert_eq!((phrase.start+phrase.length-1)%1920,0);}for pair in p.phrases.windows(2){assert!(pair[1].start-pair[0].start-pair[0].length>=4*1920);let t=Timeline::new(&s,&o);assert!(t.seconds(pair[1].start as f64)-t.seconds((pair[0].start+pair[0].length) as f64)<=30.);}assert!(p.phrases.iter().take(p.phrases.len()-1).all(|p|(p.measures-2.).abs()<0.01));}
 #[test]fn density_scales_phrase_duration(){for (hits,expected) in [(2,4.),(8,2.),(32,1.)]{let(s,o,n)=fixture(120.,32,hits);assert!((plan(&s,&o,&n,0).phrases[0].measures-expected).abs()<0.01);}}
 #[test]fn slow_tempo_conflict_is_reported_without_breaking_spacing(){let(s,o,n)=fixture(25.,48,8);let p=plan(&s,&o,&n,0);assert!(p.warnings.iter().any(|s|s.contains("30-second")));for pair in p.phrases.windows(2){assert!(pair[1].start-pair[0].start-pair[0].length>=1920*4);}}
 #[test]fn sections_meter_changes_and_global_offsets(){let(mut s,mut o,n)=fixture(120.,64,8);s.sections=vec![Section{tick:1920.*12.,name:"Solo".into()}];s.signatures=vec![Signature{tick:0.,numerator:4,denominator:4},Signature{tick:1920.*20.,numerator:3,denominator:4}];s.tempos.push(Tempo{tick:1920.*30.,bpm:180.});let a=plan(&s,&o,&n,0);assert!(a.phrases.iter().any(|p|(p.start as f64/1920.-12.).abs()<0.6||(p.start+p.length) as f64/1920.-12.<0.01&&((p.start+p.length) as f64/1920.-12.).abs()<0.6));let grid=Grid::new(&s,1920*64);for pair in a.phrases.windows(2){assert!(grid.measure(pair[1].start)-grid.measure(pair[0].start+pair[0].length)>=4.-1e-6);}o.set_chart_quarters(3);let shift=o.note_shift_ticks(&s);let moved=n.iter().map(|n|Gem{tick:n.tick+shift,..n.clone()}).collect::<Vec<_>>();let b=plan(&s,&o,&moved,0);assert_eq!(a.phrases.iter().map(|p|(p.start+shift,p.length)).collect::<Vec<_>>(),b.phrases.iter().map(|p|(p.start,p.length)).collect::<Vec<_>>());o.star_power=false;assert!(plan(&s,&o,&moved,0).phrases.is_empty());}
 #[test]fn strong_accented_chord_can_supply_the_ending(){let(mut s,o,mut n)=fixture(120.,32,8);for note in &mut s.tracks[0].notes{if note.tick==4320.{note.accent=true;note.velocity=127;}}n.push(Gem{tick:4320,lane:2,length:0});n.push(Gem{tick:4320,lane:3,length:0});let p=plan(&s,&o,&n,0);assert_eq!(p.phrases[0].start+p.phrases[0].length,4321);}
 #[test]fn sparse_passages_modifiers_and_short_endings(){let(s,o,_)=fixture(120.,32,8);let n=vec![Gem{tick:0,lane:1,length:0},Gem{tick:0,lane:66,length:0},Gem{tick:1920*25,lane:8,length:0},Gem{tick:1920*26,lane:7,length:0}];let p=plan(&s,&o,&n,0);assert!(!p.warnings.is_empty());assert_eq!(p.phrases[0].hits,1);assert!(p.phrases.iter().all(|p|n.iter().any(|n|engine::playable(n.lane)&&n.tick>=p.start&&n.tick<p.start+p.length)));}
}
