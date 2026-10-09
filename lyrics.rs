//! LRC editing, chronological repair and Clone Hero phrase generation.
use serde::{Serialize,Deserialize};
use crate::engine::Options;
#[derive(Clone,Default,Serialize,Deserialize)]
pub struct Cue{pub track:usize,pub tick:f64,pub length:f64,pub line:usize,pub text:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct Repair{pub line:usize,pub text:String,pub original_ms:i64,pub repaired_ms:i64}
#[derive(Clone,Debug)]
pub struct Entry{pub ms:i64,pub text:String,pub line:usize,pub phrase:usize,pub enhanced:bool}
#[derive(Clone,Debug)]
pub struct Document{pub entries:Vec<Entry>,pub repairs:Vec<Repair>,pub enhanced:bool,pub metadata:Vec<String>}
pub fn stamp(ms:i64)->String{let sign=if ms<0{"-"}else{""};let ms=ms.unsigned_abs();format!("{sign}{:02}:{:02}.{:03}",ms/60000,(ms/1000)%60,ms%1000)}
pub fn parse(text:&str)->Result<Document,String>{
 static TIME:std::sync::OnceLock<regex::Regex>=std::sync::OnceLock::new();
 let time=TIME.get_or_init(||regex::Regex::new(r"\[(\d+):(\d{2})(?:[.:](\d{1,3}))?\]").unwrap());
 static WORD:std::sync::OnceLock<regex::Regex>=std::sync::OnceLock::new();
 let word=WORD.get_or_init(||regex::Regex::new(r"<(\d+):(\d{2})(?:[.:](\d{1,3}))?>").unwrap());
 static OFFSET:std::sync::OnceLock<regex::Regex>=std::sync::OnceLock::new();
 let offset_re=OFFSET.get_or_init(||regex::Regex::new(r"(?i)\[offset:([+-]?\d+)\]").unwrap());
 let offset=offset_re.captures(text).and_then(|c|c[1].parse::<i64>().ok()).unwrap_or(0);
 let ms=|c:&regex::Captures<'_>|->Result<i64,String>{let min=c[1].parse::<i64>().map_err(|_|"LRC minute value is too large.")?;let sec=c[2].parse::<i64>().unwrap();if sec>=60{return Err("LRC seconds must be between 00 and 59.".into())}let frac=c.get(3).map(|m|format!("{:0<3}",m.as_str()).parse::<i64>().unwrap()).unwrap_or(0);min.checked_mul(60000).and_then(|v|v.checked_add(sec*1000+frac)).and_then(|v|v.checked_add(offset)).filter(|v|v.unsigned_abs()<86_400_000).ok_or("Lyrics timestamps must be within 24 hours.".into())};
 let mut entries=Vec::new();let mut phrase=0;let mut enhanced=false;let mut metadata=Vec::new();
 for (line_no,line) in text.trim_start_matches('\u{feff}').lines().enumerate(){let captures:Vec<_>=time.captures_iter(line).collect();if captures.is_empty(){if line.trim().starts_with('[')&&!offset_re.is_match(line){metadata.push(line.to_string());}continue}let tail=&line[captures.last().unwrap().get(0).unwrap().end()..];let words:Vec<_>=word.captures_iter(tail).collect();
  if words.is_empty(){for c in captures{let txt=tail.trim();if !txt.is_empty(){entries.push(Entry{ms:ms(&c)?,text:txt.into(),line:line_no+1,phrase,enhanced:false});phrase+=1;}}}
  else {enhanced=true;for c in &captures{let delta=ms(c)?-ms(&captures[0])?;for (i,w) in words.iter().enumerate(){let start=w.get(0).unwrap().end();let end=words.get(i+1).map(|v|v.get(0).unwrap().start()).unwrap_or(tail.len());let txt=tail[start..end].trim();if !txt.is_empty(){entries.push(Entry{ms:ms(w)?+delta,text:txt.into(),line:line_no+1,phrase,enhanced:true});}}phrase+=1;}}
 }
 if entries.is_empty(){return Err("No timed lyrics found. Open or enter a standard or enhanced LRC file.".into())}
 // Preserve the written lyric order while repairing runs of backward enhanced timestamps.
 let mut repairs=Vec::new();if enhanced{let mut i=1;while i<entries.len(){if entries[i].ms<entries[i-1].ms{let previous=entries[i-1].ms;let mut next=i+1;while next<entries.len()&&entries[next].ms<=previous{next+=1;}if next==entries.len(){return Err(format!("Backward timestamps at line {} have no later lyric to anchor the repair. Add a following lyric with a later timestamp.",entries[i].line))}let count=next-i;let gap=entries[next].ms-previous;if gap<=count as i64{return Err("There is not enough time between the surrounding lyrics to place the backward timestamps. Move the following lyric later.".into())}for j in i..next{let repaired=previous+gap*(j-i+1) as i64/(count+1) as i64;repairs.push(Repair{line:entries[j].line,text:entries[j].text.clone(),original_ms:entries[j].ms,repaired_ms:repaired});entries[j].ms=repaired;}i=next;}else{i+=1;}}}
 entries.sort_by_key(|e|e.ms);entries.dedup_by(|a,b|a.ms==b.ms&&a.text==b.text&&a.phrase==b.phrase);
 Ok(Document{entries,repairs,enhanced,metadata})
}
pub fn load(o:&Options)->Result<Document,String>{let text=if let Some(text)=&o.lyric_text{text.clone()}else{let p=o.lyrics.as_ref().ok_or("Open an LRC file to add synchronized lyrics.")?;std::fs::read_to_string(p).map_err(|e|format!("Cannot read lyrics: {e}"))?};parse(&text)}
impl Document {
 pub fn lrc(&self,offset:i64)->String{let mut groups:std::collections::BTreeMap<usize,Vec<&Entry>>=std::collections::BTreeMap::new();for e in &self.entries{groups.entry(e.phrase).or_default().push(e);}let mut lines=Vec::new();for group in groups.values(){let start=(group[0].ms+offset).max(0);let mut line=format!("[{}]",stamp(start));if group[0].enhanced{for e in group{line.push_str(&format!("<{}>{} ",stamp((e.ms+offset).max(0)),e.text));}}else{line.push_str(&group[0].text);}lines.push((start,line.trim_end().to_string()));}lines.sort_by_key(|p|p.0);{let mut out=self.metadata.iter().map(|l|l.clone()+"\n").collect::<String>();out.extend(lines.into_iter().map(|(_,l)|l+"\n"));out}}
 pub fn review(&self,history:&[Repair])->Vec<Repair>{let mut out=history.to_vec();for r in &self.repairs{if !out.contains(r){out.push(r.clone());}}out}
}
pub fn from_guitar_pro(score:&crate::engine::Score,o:&Options,track:usize,verse:usize)->Result<String,String>{
 let timeline=crate::timing::Timeline::new(score,o);let mut cues:Vec<_>=score.vocal_lyrics.iter().filter(|c|c.track==track&&c.line==verse).collect();cues.sort_by(|a,b|a.tick.total_cmp(&b.tick));if cues.is_empty(){return Err("This track/verse contains no Guitar Pro lyrics.".into())}let mut out=String::new();let mut text=String::new();let mut start=0.;let mut end=0.;let mut join=false;
 for c in cues{if !text.is_empty()&&(c.tick-end>=score.ppq as f64||c.tick-start>=score.ppq as f64*8.){let ms=(timeline.seconds(start+o.note_shift_ticks(score) as f64)*1000.).round() as i64;out.push_str(&format!("[{}]{}\n",stamp(ms),text));text.clear();join=false;}if text.is_empty(){start=c.tick;}if !text.is_empty()&&!join{text.push(' ')}join=c.text.ends_with('-');text.push_str(c.text.trim_end_matches('-'));end=c.tick+c.length;}
 if !text.is_empty(){let ms=(timeline.seconds(start+o.note_shift_ticks(score) as f64)*1000.).round() as i64;out.push_str(&format!("[{}]{}\n",stamp(ms),text));}Ok(out)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn backward_runs_are_evenly_spaced_without_reordering_words(){let d=parse("[00:10.000]<00:10.000>A <00:08.000>B <00:07.000>C <00:16.000>D").unwrap();assert_eq!(d.entries.iter().map(|e|e.ms).collect::<Vec<_>>(),vec![10000,12000,14000,16000]);assert_eq!(d.entries.iter().map(|e|e.text.as_str()).collect::<Vec<_>>(),vec!["A","B","C","D"]);assert_eq!(d.repairs.len(),2);assert_eq!(d.repairs[0].original_ms,8000);assert_eq!(d.repairs[0].repaired_ms,12000);assert!(parse(&d.lrc(0)).unwrap().repairs.is_empty());}
 #[test]fn cross_line_repairs_use_the_next_lyric_and_preserve_phrases(){let d=parse("[00:10]<00:10>A <00:08>B\n[00:14]<00:14>C <00:15>D").unwrap();assert_eq!(d.entries[1].ms,12000);assert_eq!(d.entries[1].phrase,0);assert_eq!(d.entries[2].phrase,1);assert!(d.lrc(500).contains("<00:12.500>B"));}
 #[test]fn standard_multiple_timestamps_offsets_and_order_are_preserved(){let d=parse("[offset:100]\n[00:03.000][00:01.50]Hello").unwrap();assert_eq!(d.entries.iter().map(|e|e.ms).collect::<Vec<_>>(),vec![1600,3100]);assert!(d.repairs.is_empty());assert!(!d.enhanced);}
 #[test]fn missing_next_anchor_and_too_small_gaps_are_actionable(){assert!(parse("[00:10]<00:10>A <00:08>B").unwrap_err().contains("no later lyric"));assert!(parse("[00:10.000]<00:10.000>A <00:08.000>B <00:07.000>C <00:10.001>D").unwrap_err().contains("not enough time"));}
}
