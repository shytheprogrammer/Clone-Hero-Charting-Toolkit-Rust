//! A shared, scaled source tempo map for generation, playback, lyrics and export.
use crate::engine::{Options, Score};
pub struct Timeline { pub tempos: Vec<(i64,f64)>, pub signatures:Vec<(i64,i32,i32)>,ppq:f64 }
impl Timeline {
    pub fn new(score:&Score, options:&Options)->Self {
        let source=score.tempos.iter().find(|t|t.bpm.is_finite()&&t.bpm>0.).map(|t|t.bpm).unwrap_or(options.bpm);
        let shift=options.note_shift_ticks(score);
        let mut points=std::collections::BTreeMap::from([(0,options.bpm)]);
        for tempo in score.tempos.iter().filter(|_|!options.constant_bpm) {
            if tempo.bpm.is_finite()&&tempo.bpm>0.&&tempo.tick.is_finite(){points.insert((tempo.tick.round() as i64+shift).max(0),tempo.bpm/source*options.bpm);}
        }
        let mut tempos=Vec::new();
        for (tick,bpm) in points {if tempos.last().is_none_or(|&(_,last)|last!=bpm){tempos.push((tick,bpm));}}
        let first=score.signatures.first().map(|s|(s.numerator,s.denominator)).unwrap_or((4,4));let mut meters=std::collections::BTreeMap::from([(0,first)]);
        for s in &score.signatures{if s.numerator>0&&s.denominator>0&&(s.denominator as u32).is_power_of_two(){meters.insert((s.tick.round() as i64+shift).max(0),(s.numerator,s.denominator));}}
        let mut signatures=Vec::new();for (tick,(n,d)) in meters{if signatures.last().is_none_or(|&(_,pn,pd)|pn!=n||pd!=d){signatures.push((tick,n,d));}}
        Self{tempos,signatures,ppq:score.ppq as f64}
    }
    pub fn seconds(&self,tick:f64)->f64 {
        let tick=tick.max(0.);let mut seconds=0.;
        for (i,&(start,bpm)) in self.tempos.iter().enumerate(){if tick<=start as f64{break;}let end=self.tempos.get(i+1).map(|p|p.0 as f64).unwrap_or(tick).min(tick);seconds+=(end-start as f64)/self.ppq*60./bpm;}
        seconds
    }
    pub fn tick(&self,seconds:f64)->f64 {
        let mut remaining=seconds.max(0.);
        for (i,&(start,bpm)) in self.tempos.iter().enumerate(){let duration=self.tempos.get(i+1).map(|p|(p.0-start) as f64/self.ppq*60./bpm).unwrap_or(f64::INFINITY);if remaining<=duration{return start as f64+remaining*bpm/60.*self.ppq;}remaining-=duration;}
        0.
    }
    pub fn bpm(&self,tick:i64)->f64 {self.tempos.iter().rev().find(|p|p.0<=tick).map(|p|p.1).unwrap_or(self.tempos[0].1)}
    pub fn position(&self,tick:i64)->(i64,f64){let mut measure=1;for (i,&(start,n,d)) in self.signatures.iter().enumerate(){let beat=self.ppq*4./d as f64;let bar=beat*n as f64;let end=self.signatures.get(i+1).map(|s|s.0).unwrap_or(i64::MAX);if tick<end{return (measure+((tick-start).max(0) as f64/bar).floor() as i64,((tick-start).max(0) as f64%bar)/beat+1.);}measure+=((end-start) as f64/bar).ceil() as i64;} (measure,1.)}
}
#[cfg(test)] mod tests {
    use super::*;
    #[test]fn constant_mode_ignores_all_source_changes_even_after_note_shift(){let s=Score{ppq:480,tempos:vec![crate::engine::Tempo{tick:0.,bpm:120.},crate::engine::Tempo{tick:960.,bpm:240.},crate::engine::Tempo{tick:1920.,bpm:75.}],signatures:vec![crate::engine::Signature{tick:0.,numerator:3,denominator:4}],..Default::default()};let o=Options{bpm:90.,constant_bpm:true,chart_shift_bars:2,..Default::default()};let t=Timeline::new(&s,&o);assert_eq!(t.tempos,vec![(0,90.)]);assert_eq!(t.signatures,vec![(0,3,4)]);assert!((t.seconds(1920.)-8./3.).abs()<1e-9);assert!((t.tick(8./3.)-1920.).abs()<1e-9);assert_eq!(t.bpm(10000),90.);}
    #[test] fn source_tempo_map_scales_and_integrates_across_changes(){let s=Score{ppq:480,tempos:vec![crate::engine::Tempo{tick:0.,bpm:120.},crate::engine::Tempo{tick:960.,bpm:240.}],..Default::default()};let mut o=Options::default();let t=Timeline::new(&s,&o);assert_eq!(t.seconds(1920.),1.5);assert_eq!(t.tick(1.5),1920.);o.bpm=60.;let slow=Timeline::new(&s,&o);assert_eq!(slow.seconds(1920.),3.);o.chart_shift_bars=1;let shifted=Timeline::new(&s,&o);assert_eq!(shifted.tempos,vec![(0,60.),(2880,120.)]);}
}
