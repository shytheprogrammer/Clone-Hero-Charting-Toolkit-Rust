use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap,BTreeSet}, fs, path::{Path,PathBuf}, process::Command};

#[derive(Clone,Default,Deserialize,Serialize)]
pub struct Note {
    pub tick:f64, pub pitch:i32, #[serde(default)] pub length:f64, #[serde(default)] pub chart_open:bool,
    #[serde(default)] pub velocity:i32, #[serde(default)] pub hopo:bool,
    #[serde(default)] pub tap:bool, #[serde(default)] pub ghost:bool,
    #[serde(default)] pub accent:bool, #[serde(default)] pub fret:Option<i32>,
    #[serde(default)] pub dead:bool, #[serde(default)] pub palm_mute:bool,
    #[serde(default)] pub staccato:bool,
}
#[derive(Clone,Default,Deserialize,Serialize)]
pub struct Track { pub index:usize, pub name:String, pub percussion:bool, pub notes:Vec<Note> }
#[derive(Clone,Default,Deserialize,Serialize)]
pub struct Tempo { pub tick:f64,pub bpm:f64 }
#[derive(Clone,Default,Deserialize,Serialize)]
pub struct Section {pub tick:f64,pub name:String}
#[derive(Clone,Default,Deserialize,Serialize)]
pub struct Signature {pub tick:f64,pub numerator:i32,pub denominator:i32}
#[derive(Clone,Default,Deserialize,Serialize)]
pub struct Score {
    #[serde(default)] pub vocal_lyrics:Vec<crate::lyrics::Cue>,
    #[serde(default)] pub title:String, #[serde(default)] pub artist:String,
    #[serde(default)] pub album:String, pub ppq:i64, pub tracks:Vec<Track>,
    pub tempos:Vec<Tempo>, pub end:f64, #[serde(default)] pub sections:Vec<Section>,
    #[serde(default)] pub signatures:Vec<Signature>,
}
#[derive(Clone,Serialize,Deserialize)]
pub struct Options {
    pub title:String,pub artist:String,pub album:String,pub genre:String,pub year:String,
    pub charter:String,pub bpm:f64,pub offset:i32,pub lyric_offset:i32,
    pub tracks:[Option<usize>;4],pub ratings:[i32;4],pub overall:i32,
    pub difficulties:[bool;4],pub star_power:bool,pub dynamics:bool,pub strict:bool,
    pub open_bass:bool,pub double_bass:bool,pub drum_map:BTreeMap<i32,i32>,
    pub audio:Option<PathBuf>,pub lyrics:Option<PathBuf>,pub cover:Option<PathBuf>,
    pub extras:BTreeMap<String,PathBuf>,pub output:PathBuf,
    pub preview_start:i32,pub loading_phrase:String,pub icon:String,pub video_loop:bool,
    #[serde(default)] pub audio_shift_ms: i64,
    #[serde(default)] pub chart_shift_bars: i32,
    #[serde(default)] pub chart_shift_quarters:i32,
    #[serde(default="default_preview_beats")] pub pattern_preview_beats:f32,
    #[serde(default="default_preview_beats")] pub lyric_preview_beats:f32,
    #[serde(default)] pub constant_bpm: bool,
    #[serde(default)] pub lyric_text:Option<String>,
    #[serde(default)] pub gp_lyric_track:Option<usize>,
    #[serde(default)] pub gp_lyric_verse:usize,
    #[serde(default)] pub lyric_repairs:Vec<crate::lyrics::Repair>,
}
impl Default for Options {
    fn default()->Self {Self {title:String::new(),artist:String::new(),album:String::new(),genre:String::new(),year:String::new(),charter:"Charting Toolkit".into(),bpm:120.,offset:0,lyric_offset:0,tracks:[None;4],ratings:[-1;4],overall:-1,difficulties:[true;4],star_power:true,dynamics:false,strict:true,open_bass:true,double_bass:true,drum_map:(35..=81).map(|p|(p,default_drum(p))).collect(),audio:None,lyrics:None,cover:None,extras:BTreeMap::new(),output:PathBuf::new(),preview_start:0,loading_phrase:String::new(),icon:String::new(),video_loop:false,audio_shift_ms:0,chart_shift_bars:0,chart_shift_quarters:0,pattern_preview_beats:16.,lyric_preview_beats:16.,constant_bpm:false,lyric_text:None,gp_lyric_track:None,gp_lyric_verse:0,lyric_repairs:Vec::new()}}
}
fn default_preview_beats()->f32{16.}
impl Options {
    /// Migrate the old chart-delay value into the physical audio timeline once,
    /// so saved projects keep their relative note/audio alignment after baking.
    pub fn audio_alignment_ms(&self) -> i64 { self.audio_shift_ms.saturating_sub(self.offset as i64) }
    pub fn set_audio_alignment_ms(&mut self, value: i64) { self.audio_shift_ms=value.clamp(-3_600_000,3_600_000)+self.offset as i64; }
    pub fn total_chart_quarters(&self)->i32{self.chart_shift_bars*4+self.chart_shift_quarters}
    pub fn set_chart_quarters(&mut self,value:i32){let value=value.clamp(-40000,40000);self.chart_shift_bars=value.div_euclid(4);self.chart_shift_quarters=value.rem_euclid(4);}
    pub fn note_shift_ticks(&self, score: &Score) -> i64 { self.total_chart_quarters() as i64 * bar_ticks(score)/4 }
}
pub fn bar_ticks(score:&Score)->i64{score.signatures.first().filter(|s|s.numerator>0&&s.denominator>0).map(|s|score.ppq*s.numerator as i64*4/s.denominator as i64).unwrap_or(score.ppq*4).max(1)}
pub fn chart_end_tick(score:&Score,o:&Options)->i64 { (score.end.round() as i64+o.note_shift_ticks(score)).max(0) }
#[cfg(test)]
pub fn minimum_chart_shift_bars(score:&Score,o:&Options)->i32 {minimum_chart_shift_quarters(score,o)/4}
pub fn minimum_chart_shift_quarters(score:&Score,o:&Options)->i32 {
    let earliest=score.tracks.iter().filter(|t|o.tracks.contains(&Some(t.index)))
        .flat_map(|t|t.notes.iter().filter(move|n|!t.percussion||o.drum_map.get(&n.pitch).is_some_and(|lane|*lane>=0)))
        .map(|n|n.tick.round() as i64).min().unwrap_or(0);
    (-(earliest*4/bar_ticks(score))) as i32
}
fn audio_name(o:&Options)->String {
    if o.audio.is_some()&&o.audio_alignment_ms()!=0 {return "song.wav".into();}
    o.audio.as_ref().and_then(|p|p.extension()).and_then(|e|e.to_str()).map(|ext|format!("song.{}",ext.to_lowercase())).unwrap_or("song.wav".into())
}
pub const ROLES:[&str;4]=["Drums","Guitar","Bass","Rhythm"];
pub const TIERS:[&str;4]=["Easy","Medium","Hard","Expert"];
pub fn default_drum(p:i32)->i32 {match p {35|36=>0,37|38|40=>1,48|50=>2,45|47=>3,41|43=>4,42|46=>66,51|53|59=>67,49|52|55|57=>68,_=>-1}}
pub fn asset_root()->PathBuf {
    let exe=std::env::current_exe().unwrap_or_default();let p=exe.parent().unwrap_or(Path::new("."));
    if p.join("gp_bridge.cjs").exists(){return p.to_owned();}let resources=p.parent().unwrap_or(p).join("Resources");if resources.join("gp_bridge.cjs").exists(){return resources;}PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
pub fn read_score(path:&Path)->Result<Score,String> {
    let meta=fs::metadata(path).map_err(|e|e.to_string())?;
    if meta.len()>64*1024*1024 {return Err("Choose a score smaller than 64 MB.".into())}
    let root=asset_root();let bundled=root.join(if cfg!(windows){"vendor/node.exe"}else{"vendor/node"});
    let mut cmd=Command::new(if bundled.exists(){bundled}else{PathBuf::from("node")});
    cmd.arg(root.join("gp_bridge.cjs")).arg(path);
    #[cfg(windows)] { use std::os::windows::process::CommandExt;cmd.creation_flags(0x08000000); }
    let out=cmd.output().map_err(|e|format!("Could not start Guitar Pro importer: {e}"))?;
    if !out.status.success(){return Err(String::from_utf8_lossy(&out.stderr).into_owned())}
    let score:Score=serde_json::from_slice(&out.stdout).map_err(|e|e.to_string())?;
    if score.ppq<=0 || score.tracks.is_empty(){return Err("The score has no usable tracks.".into())}
    if score.tracks.iter().flat_map(|t|&t.notes).any(|n| !n.tick.is_finite() || n.tick<0. || !n.length.is_finite() || n.length<0.) {return Err("The score contains invalid note positions.".into())}
    Ok(score)
}
#[derive(Clone,Debug,Serialize)]
pub struct Gem {pub tick:i64,pub lane:i32,pub length:i64}
fn groups(track:&Track)->BTreeMap<i64,Vec<&Note>> {
    let mut g=BTreeMap::new();for n in &track.notes {g.entry(n.tick.round() as i64).or_insert_with(Vec::new).push(n);}g
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DrumCollision {
    pub tick: i64,
    pub lane: i32,
    pub pitches: Vec<i32>,
}

/// Inspect the full Expert source timeline using the same rounded ticks and
/// physical lanes as export. A cymbal and tom on one pad still collide.
pub fn drum_collisions(score: &Score, options: &Options) -> Vec<DrumCollision> {
    let Some(track) = score.tracks.iter().find(|t| {
        t.percussion && Some(t.index) == options.tracks[0]
    }) else { return Vec::new() };
    let mut mapped: BTreeMap<(i64, i32), Vec<i32>> = BTreeMap::new();
    for note in &track.notes {
        let code = *options.drum_map.get(&note.pitch).unwrap_or(&-1);
        if code < 0 { continue; }
        let lane = if code >= 64 { code - 64 } else { code };
        mapped.entry((note.tick.round() as i64+options.note_shift_ticks(score), lane)).or_default().push(note.pitch);
    }
    mapped.into_iter().filter_map(|((tick, lane), mut pitches)| {
        if pitches.len() < 2 { return None; }
        pitches.sort_unstable();
        Some(DrumCollision { tick, lane, pitches })
    }).collect()
}

pub fn gems(score:&Score,options:&Options,role:usize,tier:usize)->Result<Vec<Gem>,String> {
    let index=options.tracks[role].ok_or("Choose a track.")?;
    let track=score.tracks.iter().find(|t|t.index==index).ok_or("Track does not exist.")?;
    if track.percussion!=(role==0){return Err(format!("Select a {} track for {}.",if role==0{"percussion"}else{"pitched"},ROLES[role]))}
    if track.notes.is_empty(){return Err(format!("{} has no notes.",track.name))}
    if role>0{let out=crate::pitched::generate(score,options,track,role,tier);if out.is_empty(){return Err(format!("No playable notes remain in {}.",ROLES[role]));}if out.iter().any(|g|g.tick<0){return Err("Moving notes earlier would put a note before the start of the chart.".into());}return Ok(out);}
    let grouped=groups(track);let mut out=Vec::new();let mut last=-score.ppq;let mut last_kick=-score.ppq;let mut alt=false;

    let spacing=if role==0{[score.ppq,score.ppq/2,score.ppq/4,0][tier]}else{[score.ppq,score.ppq/2,score.ppq/4,0][tier]};
    let entries:Vec<_>=grouped.iter().collect();
    for (tick,notes) in &entries {
        let tick=**tick;if tick-last<spacing {continue}
        if role==0 {
            let mut mapped:BTreeMap<i32,(bool,bool,bool)>=BTreeMap::new();
            for n in *notes {
                let code=*options.drum_map.get(&n.pitch).unwrap_or(&-1);
                if code<0 {if options.strict && !options.drum_map.contains_key(&n.pitch){return Err(format!("Unmapped drum pitch {}.",n.pitch))}continue}
                let lane=if code>=64{code-64}else{code};
                if mapped.contains_key(&lane)&&options.strict{return Err(format!("Drum lane collision at tick {tick}. Adjust mapping or disable strict export."))}
                let entry=mapped.entry(lane).or_insert((false,false,false));entry.0|=code>=64;
                entry.1|=n.ghost || (options.dynamics && n.velocity<50);entry.2|=n.accent || (options.dynamics && n.velocity>=115);
            }
            let max_hits=[2,2,3,6][tier];
            for (lane,(cymbal,ghost,accent)) in mapped.into_iter().take(max_hits) {
                let actual=if lane==0 && tier==3 && options.double_bass && options.bpm>110. {
                    alt=if tick-last_kick<=score.ppq/4{!alt}else{false};last_kick=tick;if alt{32}else{0}
                }else{lane};
                out.push(Gem{tick,lane:actual,length:0});
                if cymbal && tier>=2{out.push(Gem{tick,lane:64+lane,length:0})}
                if lane>0 && tier==3 && (ghost||accent){out.push(Gem{tick,lane:if accent{33+lane}else{39+lane},length:0})}
            }
        }
        last=tick;
    }
    out.sort_by_key(|g|(g.tick,g.lane));out.dedup_by_key(|g|(g.tick,g.lane));
    if out.is_empty(){return Err(format!("No playable notes remain in {}.",ROLES[role]))}
    let shift=options.note_shift_ticks(score);for gem in &mut out {gem.tick+=shift;if gem.tick<0{return Err("Moving notes earlier would put a note before the start of the chart.".into());}}
    Ok(out)
}
pub fn rating(gems:&[Gem],score:&Score,options:&Options)->i32 {
    let playable:Vec<_>=gems.iter().filter(|g|g.lane<=4 || g.lane==7 || g.lane==32).collect();
    let mut original=options.clone();original.chart_shift_bars=0;original.chart_shift_quarters=0;let duration=crate::timing::Timeline::new(score,&original).seconds(score.end).max(1.);
    let density=playable.len() as f64/duration;
    [1.5,2.5,4.,5.5,7.5,10.].iter().filter(|&&v|density>=v).count() as i32
}
fn clean(s:&str)->String{s.replace(['\n','\r','\0']," ").replace('"',"'").replace('\\',"/")}
pub fn safe_name(s:&str)->String {
    let mut s:String=s.chars().map(|c|if c.is_control()||"<>:\"/\\|?*".contains(c){'_'}else{c}).take(100).collect();
    s=s.trim_matches([' ','.']).to_owned();if s.is_empty(){s="Untitled".into()}
    let base=s.split('.').next().unwrap_or("").to_uppercase();
    if ["CON","PRN","AUX","NUL"].contains(&base.as_str())||(1..=9).any(|n|base==format!("COM{n}")||base==format!("LPT{n}")){s.insert(0,'_')}s
}
#[cfg(test)]
fn parse_lrc(text:&str)->Result<Vec<crate::lyrics::Entry>,String>{crate::lyrics::parse(text).map(|d|d.entries)}
pub fn build_chart(score:&Score,o:&Options)->Result<(String,serde_json::Value),String> {
    if !o.bpm.is_finite()||o.bpm<0.001||o.bpm>1000.{return Err("BPM must be between 0.001 and 1000.".into())}
    if !(-3_600_000..=3_600_000).contains(&o.audio_alignment_ms()){return Err("Audio alignment must be within one hour.".into());}
    if !(-40000..=40000).contains(&o.total_chart_quarters()){return Err("The chart shift must be within 10,000 bars.".into());}
    if o.total_chart_quarters()<minimum_chart_shift_quarters(score,o){return Err("Moving notes earlier would put a note before the start of the chart.".into());}
    if !o.difficulties.iter().any(|v|*v){return Err("Enable at least one difficulty.".into())}
    if o.tracks.iter().all(Option::is_none){return Err("Enable at least one instrument.".into())}
    if o.ratings.iter().chain(std::iter::once(&o.overall)).any(|r|!(-1..=6).contains(r)){return Err("Ratings must be Auto or a value from 0 to 6.".into())}
    for text in [&o.title,&o.artist,&o.album,&o.genre,&o.year,&o.charter,&o.loading_phrase,&o.icon]{if text.contains(['\n','\r','\0']){return Err("Song properties must be single-line text.".into())}}
    let audio=audio_name(o);let note_shift=o.note_shift_ticks(score);let timeline=crate::timing::Timeline::new(score,o);
    let mut chart=format!("[Song]\n{{\n  Name = \"{}\"\n  Artist = \"{}\"\n  Charter = \"{}\"\n  Offset = 0\n  Resolution = {}\n  MusicStream = \"{}\"\n}}\n[SyncTrack]\n{{\n  0 = B {}\n  0 = TS 4 2\n}}\n[Events]\n{{\n",clean(&o.title),clean(&o.artist),clean(&o.charter),score.ppq,audio,(o.bpm*1000.).round());
    let sync_start=chart.find("[SyncTrack]").unwrap();let sync_end=chart.find("[Events]").unwrap();
    let mut sync=String::from("[SyncTrack]\n{\n");let mut markers:Vec<_>=timeline.tempos.iter().map(|(tick,bpm)|(*tick,format!("B {}",(bpm*1000.).round()))).collect();for (tick,n,d) in &timeline.signatures{markers.push((*tick,format!("TS {n} {}",(*d as u32).ilog2())));}markers.sort_by_key(|m|m.0);for (tick,event) in markers{sync.push_str(&format!("  {tick} = {event}\n"));}sync.push_str("}\n");chart.replace_range(sync_start..sync_end,&sync);
    let mut events:Vec<(i64,String)>=score.sections.iter().map(|s|(s.tick.round() as i64+note_shift,format!("section {}",clean(&s.name)))).filter(|e|e.0>=0).collect();
    let mut lyric_count=0;let mut adjusted=String::new();let mut end=chart_end_tick(score,o);
    let mut lyric_repairs=Vec::new();
    if o.lyrics.is_some()||o.lyric_text.is_some() {
        let doc=crate::lyrics::load(o)?;lyric_repairs=doc.review(&o.lyric_repairs);
        let shift=o.lyric_offset as i64+o.audio_alignment_ms();
        let mut phrases:BTreeMap<usize,Vec<(i64,String)>>=BTreeMap::new();
        for l in &doc.entries{let ms=(l.ms+shift).max(0);let tick=timeline.tick(ms as f64/1000.).round() as i64;phrases.entry(l.phrase).or_default().push((tick,clean(&l.text)));end=end.max(tick+score.ppq*2);lyric_count+=1;}
        let mut sorted:Vec<_>=phrases.into_values().collect();sorted.sort_by_key(|p|p[0].0);
        let mut merged:Vec<Vec<(i64,String)>>=Vec::new();for p in sorted{if let Some(last)=merged.last_mut(){if p[0].0<=last.last().unwrap().0{last.extend(p);last.sort_by_key(|p|p.0);continue;}}merged.push(p);}
        for (i,p) in merged.iter().enumerate(){let start=p[0].0;let next=merged.get(i+1).map(|p|p[0].0).unwrap_or(end.max(p.last().unwrap().0+1)+1);
            events.push((start,"phrase_start".into()));let mut words:BTreeMap<i64,Vec<&str>>=BTreeMap::new();for (t,word) in p{words.entry(*t).or_default().push(word);}for (t,words) in words{events.push((t,format!("lyric {}",words.join(" "))));}
            events.push((next-1,"phrase_end".into()));
        }
        adjusted=doc.lrc(shift);
    }
    events.sort_by_key(|e|e.0);for (tick,event) in events{chart.push_str(&format!("  {tick} = E \"{event}\"\n"))}chart.push_str(&format!("  {} = E \"end\"\n}}\n",end+1));
    let mut reports=Vec::new();let mut audits=Vec::new();
    for role in 0..4 {if o.tracks[role].is_none(){continue}let expert=gems(score,o,role,3)?;let estimated=rating(&expert,score,o);let selected=if o.ratings[role]>=0{o.ratings[role]}else{estimated};let mut counts=BTreeMap::new();
        for (tier,name) in TIERS.iter().enumerate(){if !o.difficulties[tier]{continue}let notes=gems(score,o,role,tier)?;if role>0{audits.push(serde_json::json!({"instrument":ROLES[role],"difficulty":name,"findings":crate::pitched::audit(score,o,&notes,tier)}));}let suffix=["Drums","Single","DoubleBass","DoubleRhythm"][role];chart.push_str(&format!("[{name}{suffix}]\n{{\n"));
            let mut local_events:Vec<_>=notes.iter().map(|n|(n.tick,format!("N {} {}",n.lane,n.length))).collect();
            if o.star_power{let bar=score.ppq*4;let occupied:BTreeSet<_>=notes.iter().filter(|n|n.lane<=4||n.lane==7||n.lane==32).map(|n|(n.tick-note_shift)/(bar*8)).collect();for chunk in occupied{let start=notes.iter().filter(|n|n.tick-note_shift>=chunk*bar*8&&n.tick-note_shift<(chunk+1)*bar*8).map(|n|n.tick).min().unwrap();let length=(bar*2).min((chart_end_tick(score,o)-start).max(1));local_events.push((start,format!("S 2 {length}")));}}
            local_events.sort_by_key(|e|e.0);for (tick,event) in local_events{chart.push_str(&format!("  {tick} = {event}\n"))}
            chart.push_str("}\n");counts.insert(name.to_string(),notes.iter().filter(|n|n.lane<=4||n.lane==7||n.lane==32).count());
        }
        reports.push(serde_json::json!({"role":ROLES[role],"track":o.tracks[role],"estimated_rating":estimated,"rating":selected,"counts":counts}));
    }
    let report=serde_json::json!({"lyric_repairs":lyric_repairs,"application":concat!("Charting Toolkit ",env!("CARGO_PKG_VERSION")),"instruments":reports,"charting_policy":"Clone Hero Guitar Charting Rulebook v1.0","charting_audit":audits,"constant_bpm":o.constant_bpm,"tempo_map":timeline.tempos,"time_signatures":timeline.signatures,"lyrics":lyric_count,"adjusted_lrc":adjusted,"bpm":o.bpm,"resolution":score.ppq,"end_tick":end+1,"alignment":{"audio_shift_ms":o.audio_alignment_ms(),"chart_shift_bars":o.total_chart_quarters() as f64/4.,"chart_shift_ticks":note_shift,"audio_baked":o.audio.is_some()&&o.audio_alignment_ms()!=0,"export_delay_ms":0},"options":o,"review":"Review every exported part in Moonscraper; difficulty reductions and ratings are estimates. Waveform alignment is baked into exported audio; original recordings are preserved."});
    Ok((chart,report))
}
fn lyric_review_text(report:&serde_json::Value)->String{let mut out=String::new();if let Some(repairs)=report["lyric_repairs"].as_array(){if !repairs.is_empty(){out.push_str("Automatically repaired backward LRC timestamps. Review these lyrics against the recording. Values below are file timestamps before your global lyric/audio shift.\n\n");}for r in repairs{out.push_str(&format!("Line {} · {} · {} -> {}\n",r["line"],r["text"].as_str().unwrap_or(""),crate::lyrics::stamp(r["original_ms"].as_i64().unwrap_or(0)),crate::lyrics::stamp(r["repaired_ms"].as_i64().unwrap_or(0))));}}out}
pub const EXTRA_TYPES:[(&str,&str,&str);19]=[
    ("Photo background","background","image"),("Video background","video","video"),("Preview audio","preview","audio"),
    ("Guitar stem","guitar","audio"),("Bass stem","bass","audio"),("Rhythm stem","rhythm","audio"),("Drums stem","drums","audio"),
    ("Kick stem","drums_1","audio"),("Snare stem","drums_2","audio"),("Toms stem","drums_3","audio"),("Cymbals stem","drums_4","audio"),
    ("Vocals stem","vocals","audio"),("Keys stem","keys","audio"),("Crowd audio","crowd","audio"),
    ("Highway image","Highways","custom-image"),("Highway video","Video Highways","custom-video"),
    ("Highway video config","Video Highways","config"),("Song icon image","Game Icons","custom-image"),("Color profile","Colors","config")];
fn extension(path:&Path)->String{path.extension().and_then(|v|v.to_str()).unwrap_or("").to_lowercase()}
pub fn allowed(kind:&str)->&'static [&'static str]{match kind{"image"|"custom-image"=>&["png","jpg","jpeg"],"video"=>&["mp4","avi","webm","ogv","mpeg"],"custom-video"=>&["webm"],"config"=>&["ini"],_=>&["ogg","opus","mp3","wav"]}}
fn check_file(path:&Path,kind:&str)->Result<(),String>{if !path.is_file()||!allowed(kind).contains(&extension(path).as_str()){Err(format!("Choose an existing {kind} file: {}",path.display()))}else{Ok(())}}
pub fn export(score:&Score,o:&Options)->Result<PathBuf,String> {
    let (chart,mut report)=build_chart(score,o)?;
    if o.output.as_os_str().is_empty()||!o.output.is_dir(){return Err("Choose an existing output folder.".into())}
    if let Some(p)=&o.audio{check_file(p,"audio")?}if let Some(p)=&o.cover{check_file(p,"image")?}
    if o.extras.contains_key("Drums stem")&&["Kick stem","Snare stem","Toms stem","Cymbals stem"].iter().any(|k|o.extras.contains_key(*k)){return Err("Choose either a combined drum stem or separate drum stems.".into())}
    if o.extras.contains_key("Highway video config")&&!o.extras.contains_key("Highway video"){return Err("A highway video config needs its matching highway video.".into())}
    for (key,p) in &o.extras{let (_,_,kind)=EXTRA_TYPES.iter().find(|(k,_,_)|k==key).ok_or("Unknown extra type")?;check_file(p,kind)?;}
    let name=safe_name(&format!("{} - {}",o.artist,o.title));let mut target=o.output.join(&name);let mut n=2;
    while target.exists(){target=o.output.join(format!("{name} ({n})"));n+=1;}
    // Reserve a unique staging directory; only publish the complete package.
    let stage=o.output.join(format!(".chart-starter-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    fs::create_dir(&stage).map_err(|e|e.to_string())?;
    let result=(||->Result<(),String>{
        fs::write(stage.join("notes.chart"),chart).map_err(|e|e.to_string())?;
        let shift=o.audio_alignment_ms();
        let mut song_duration=crate::timing::Timeline::new(score,o).seconds(report["end_tick"].as_i64().unwrap_or(chart_end_tick(score,o)) as f64);
        if let Some(audio)=&o.audio {
            let duration=if shift!=0 {crate::audio::write_aligned(audio,&stage.join("song.wav"),shift)?}else{
                let duration=crate::audio::duration(audio)?;
                fs::copy(audio,stage.join(audio_name(o))).map_err(|e|e.to_string())?;duration
            };
            song_duration=song_duration.max(duration);
            report["alignment"]["export_audio_duration_seconds"]=serde_json::json!(duration);
        }else{
            let seconds=(song_duration+3.).max(1.);if seconds>21600.{return Err("Practice audio is limited to six hours.".into())}
            let mut wav=hound::WavWriter::create(stage.join("song.wav"),hound::WavSpec{channels:1,sample_rate:8000,bits_per_sample:16,sample_format:hound::SampleFormat::Int}).map_err(|e|e.to_string())?;
            for _ in 0..(seconds*8000.).ceil() as usize{wav.write_sample(0i16).map_err(|e|e.to_string())?}wav.finalize().map_err(|e|e.to_string())?;
        }
        let instruments=report["instruments"].as_array().unwrap();let overall=if o.overall>=0{o.overall}else{(instruments.iter().map(|r|r["rating"].as_i64().unwrap_or(0)).sum::<i64>() as f64/instruments.len().max(1) as f64).round() as i32};
        let mut ini=format!("[song]\nname = {}\nartist = {}\nalbum = {}\ngenre = {}\nyear = {}\ncharter = {}\ndelay = 0\npreview_start_time = {}\nsong_length = {}\ndiff_band = {}\nloading_phrase = {}\nicon = {}\nvideo_loop = {}\n",o.title,o.artist,o.album,o.genre,o.year,o.charter,(o.preview_start as i64+shift).max(0),(song_duration*1000.).ceil() as i64,overall,o.loading_phrase,o.icon,if o.video_loop{"True"}else{"False"});
        for (i,r) in instruments.iter().enumerate(){let role=r["role"].as_str().unwrap();let key=match role{"Drums"=>"drums","Guitar"=>"guitar","Bass"=>"bass",_=>"rhythm"};ini.push_str(&format!("diff_{key} = {}\n",instruments[i]["rating"]));if role=="Drums"{ini.push_str("pro_drums = True\nfive_lane_drums = False\n")}}
        fs::write(stage.join("song.ini"),ini).map_err(|e|e.to_string())?;
        fs::write(stage.join("conversion-report.json"),serde_json::to_vec_pretty(&report).unwrap()).map_err(|e|e.to_string())?;
        let mut audit=String::from("Charting review — Clone Hero Guitar Charting Rulebook v1.0\nSpacing targets are authoring recommendations, not game limits. Verify attacks, releases and techniques against your recording; playtest every difficulty.\n\n");
        for part in report["charting_audit"].as_array().unwrap(){audit.push_str(&format!("{} / {}\n",part["instrument"].as_str().unwrap(),part["difficulty"].as_str().unwrap()));let findings=part["findings"].as_array().unwrap();if findings.is_empty(){audit.push_str("No automated spacing warnings. Musical review is still required.\n");}for f in findings{audit.push_str(&format!("{} — {}:{:.3} · {:.2} ms · {:.2} BPM · {}\n  {}\n",f["severity"].as_str().unwrap(),f["measure"],f["beat"].as_f64().unwrap(),f["delta_ms"].as_f64().unwrap(),f["bpm"].as_f64().unwrap(),f["pattern"].as_str().unwrap(),f["suggested_fix"].as_str().unwrap()));}audit.push('\n');}
        if !lyric_review_text(&report).is_empty(){fs::write(stage.join("LYRIC-REVIEW.txt"),lyric_review_text(&report)).map_err(|e|e.to_string())?;}
        fs::write(stage.join("CHARTING-REVIEW.txt"),audit).map_err(|e|e.to_string())?;
        if let Some(p)=&o.cover{let ext=extension(p);fs::copy(p,stage.join(format!("album.{}",if ext=="jpeg"{"jpg"}else{&ext}))).map_err(|e|e.to_string())?;}
        if report["lyrics"].as_u64().unwrap_or(0)>0{fs::write(stage.join("lyrics.lrc"),report["adjusted_lrc"].as_str().unwrap_or("")).map_err(|e|e.to_string())?;}
        for (key,p) in &o.extras{let (_,stem,kind)=EXTRA_TYPES.iter().find(|(k,_,_)|k==key).unwrap();let ext=extension(p);let ext=if ext=="jpeg"{"jpg"}else{&ext};
            if *kind=="audio"&&key!="Preview audio"&&shift!=0 {crate::audio::write_aligned(p,&stage.join(format!("{stem}.wav")),shift)?;continue;}
            let dest=if key=="Highway video"||key=="Highway video config"{stage.join("Extras/Custom/Video Highways/Chart highway").join(if key=="Highway video"{"highway.webm"}else{"config.ini"})}
            else if kind.starts_with("custom")||*kind=="config"{stage.join("Extras/Custom").join(stem).join(p.file_name().unwrap())}
            else{stage.join(format!("{stem}.{ext}"))};fs::create_dir_all(dest.parent().unwrap()).map_err(|e|e.to_string())?;fs::copy(p,dest).map_err(|e|e.to_string())?;
        }
        if stage.join("Extras").exists(){fs::write(stage.join("Extras/INSTALL.txt"),"Copy Extras/Custom contents into Clone Hero's Custom folder. Review existing files before replacing them. Rescan custom content and select your highway/colors in game. Highway video should use VP8 WebM. Full-length audio stems receive the same baked alignment as the main recording. Preview excerpts are kept intact. Use synchronized stems and a backing-only main mix.\n").map_err(|e|e.to_string())?;}
        fs::write(stage.join("REVIEW.txt"),"Open notes.chart in Moonscraper and review all instruments, difficulties, lyrics, dynamics and Star Power before playing. Audio alignment is baked into the new main recording and full-length stems, with song.ini delay zero. Original files are preserved. One shared bar offset moves all instruments and difficulties, plus sections and Star Power. Source tempo changes and time signatures are preserved; Overall BPM scales the tempo map. Review CHARTING-REVIEW.txt for pitched-instrument findings. Automatic fret mappings and reductions are estimates. A silent practice WAV is generated when no recording is selected.\n").map_err(|e|e.to_string())?;
        fs::rename(&stage,&target).map_err(|e|format!("Could not publish the completed song folder: {e}"))?;Ok(())
    })();if result.is_err(){let _=fs::remove_dir_all(&stage);}result?;Ok(target)
}
pub fn smoke_test()->Result<(),String>{
    let root=asset_root();for file in ["demo.gp","demo-fretted.gp","demo-vocals.gp"]{let mut score=read_score(&root.join("examples").join(file))?;let mut o=Options{title:score.title.clone(),artist:score.artist.clone(),album:score.album.clone(),bpm:score.tempos.first().map(|t|t.bpm).unwrap_or(120.),strict:false,..Default::default()};
        o.tracks[0]=score.tracks.iter().find(|t|t.percussion).map(|t|t.index);for role in 1..4{o.tracks[role]=score.tracks.iter().filter(|t|!t.percussion).nth(role-1).or_else(||score.tracks.iter().find(|t|!t.percussion)).map(|t|t.index)}
        if file=="demo-vocals.gp"{let drums=read_score(&root.join("examples/demo.gp"))?;let mut track=drums.tracks.into_iter().find(|t|t.percussion).unwrap();track.index=3;score.tracks.push(track);o.tracks=[Some(3),Some(0),Some(1),Some(0)];o.lyrics=Some(root.join("examples/demo-lyrics.lrc"));}
        if file=="demo-fretted.gp"{o.lyrics=Some(root.join("examples/demo.lrc"));}
        o.output=root.join("smoke-output");fs::create_dir_all(&o.output).map_err(|e|e.to_string())?;let path=export(&score,&o)?;
        let chart=fs::read_to_string(path.join("notes.chart")).map_err(|e|e.to_string())?;
        for role in 0..4{if o.tracks[role].is_some(){for tier in TIERS{if !chart.contains(&format!("[{tier}{}]",["Drums","Single","DoubleBass","DoubleRhythm"][role])){return Err("An exported instrument section is missing.".into())}}}}
        let wave=hound::WavReader::open(path.join("song.wav")).map_err(|e|e.to_string())?;if wave.duration()==0{return Err("Practice audio is empty.".into())}
        if o.lyrics.is_some() && (!chart.contains("lyric ")||!path.join("lyrics.lrc").exists()){return Err("Lyrics were not exported.".into())}
        println!("PASS: {} → {}",file,path.display());
    }Ok(())
}
#[cfg(test)] mod tests {
    use super::*;
    #[test]fn quarter_bar_shift_and_preview_settings_preserve_export_timing(){
        let mut s=score();let mut pitched=s.tracks[0].clone();pitched.index=1;pitched.percussion=false;s.tracks.push(pitched);let mut o=Options{tracks:[Some(0),Some(1),Some(1),Some(1)],strict:false,..Default::default()};
        let original=build_chart(&s,&o).unwrap().0;
        o.pattern_preview_beats=4.;o.lyric_preview_beats=32.;assert_eq!(original,build_chart(&s,&o).unwrap().0);
        o.set_chart_quarters(1);assert_eq!(o.note_shift_ticks(&s),bar_ticks(&s)/4);
        for role in 0..4{for tier in 0..4{let shifted=gems(&s,&o,role,tier).unwrap();let mut zero=o.clone();zero.set_chart_quarters(0);let baseline=gems(&s,&zero,role,tier).unwrap();assert_eq!(shifted.len(),baseline.len());for (a,b) in shifted.iter().zip(&baseline){assert_eq!(a.tick,b.tick+bar_ticks(&s)/4);}}}
        o.set_chart_quarters(-1);assert_eq!(o.total_chart_quarters(),-1);assert!(build_chart(&s,&o).is_err());
        let mut old=serde_json::to_value(Options::default()).unwrap();for field in ["chart_shift_quarters","pattern_preview_beats","lyric_preview_beats"]{old.as_object_mut().unwrap().remove(field);}
        let loaded:Options=serde_json::from_value(old).unwrap();assert_eq!(loaded.total_chart_quarters(),0);assert_eq!(loaded.pattern_preview_beats,16.);assert_eq!(loaded.lyric_preview_beats,16.);
    }
    #[test]fn export_tempo_meter_lyrics_and_global_shift_share_one_timeline(){
        let mut s=score();s.tracks[0].percussion=false;for n in &mut s.tracks[0].notes{n.pitch=60;}
        s.tempos=vec![Tempo{tick:0.,bpm:120.},Tempo{tick:960.,bpm:240.}];s.signatures=vec![Signature{tick:0.,numerator:3,denominator:4},Signature{tick:1440.,numerator:4,denominator:4}];
        let mut o=Options{tracks:[None,Some(0),Some(0),Some(0)],lyrics:Some(asset_root().join("examples/demo.lrc")),..Default::default()};
        let (chart,report)=build_chart(&s,&o).unwrap();assert!(chart.contains("960 = B 240000"));assert!(chart.contains("0 = TS 3 2"));assert!(chart.contains("1440 = TS 4 2"));assert!(chart.contains("2880 = E \"lyric With a beat\""));assert_eq!(report["charting_audit"].as_array().unwrap().len(),12);
        let sync=chart.split("[SyncTrack]").nth(1).unwrap().split("[Events]").next().unwrap();let ticks:Vec<i64>=sync.lines().filter_map(|l|l.trim().split_once(" = ").and_then(|(t,_)|t.parse().ok())).collect();assert!(ticks.windows(2).all(|p|p[0]<=p[1]));
        assert_eq!(bar_ticks(&s),1440);o.chart_shift_bars=1;let (shifted,_)=build_chart(&s,&o).unwrap();assert!(shifted.contains("2400 = B 240000"));assert!(shifted.contains("2880 = TS 4 2"));for role in 1..4{for tier in 0..4{assert_eq!(gems(&s,&o,role,tier).unwrap()[0].tick,1440);}}
        o.chart_shift_bars=0;o.bpm=60.;let (slow,_)=build_chart(&s,&o).unwrap();assert!(slow.contains("960 = B 120000"));assert!(slow.contains("960 = E \"lyric With a beat\""));
        o.constant_bpm=true;o.bpm=123.456;let (constant,report)=build_chart(&s,&o).unwrap();let flags=constant.lines().filter(|line|line.contains(" = B ")).collect::<Vec<_>>();assert_eq!(flags,vec!["  0 = B 123456"]);assert_eq!(report["tempo_map"],serde_json::json!([[0,123.456]]));assert_eq!(report["constant_bpm"],true);assert!(constant.contains("0 = TS 3 2"));assert!(constant.contains("1440 = TS 4 2"));assert!(constant.contains("1975 = E \"lyric With a beat\""));
        let notes=gems(&s,&o,1,3).unwrap();let constant_rating=rating(&notes,&s,&o);let flat=Score{tempos:vec![],..s.clone()};assert_eq!(constant_rating,rating(&notes,&flat,&o));
        o.chart_shift_bars=1;let (shifted,report)=build_chart(&s,&o).unwrap();assert_eq!(shifted.lines().filter(|line|line.contains(" = B ")).collect::<Vec<_>>(),vec!["  0 = B 123456"]);assert_eq!(report["tempo_map"],serde_json::json!([[0,123.456]]));
        o.constant_bpm=false;let (restored,_)=build_chart(&s,&o).unwrap();assert_eq!(restored.lines().filter(|line|line.contains(" = B ")).count(),2);assert!(restored.contains("2400 = B 246912"));
    }
    #[test]
    fn bar_shift_translates_every_role_tier_section_and_star_power() {
        let mut s=score();s.sections=vec![Section{tick:1920.,name:"Verse".into()}];
        let mut pitched=s.tracks[0].clone();pitched.index=1;pitched.percussion=false;
        for (i,n) in pitched.notes.iter_mut().enumerate(){n.pitch=60+(i%7) as i32;n.length=240.;n.hopo=i%2==0;}
        s.tracks.push(pitched);
        let mut o=Options{tracks:[Some(0),Some(1),Some(1),Some(1)],..Default::default()};
        let before:Vec<_>=(0..4).flat_map(|role|(0..4).map(move|tier|(role,tier))).map(|(r,t)|gems(&s,&o,r,t).unwrap()).collect();
        let (original,_)=build_chart(&s,&o).unwrap();
        o.chart_shift_bars=3;
        for ((r,t),expected) in (0..4).flat_map(|r|(0..4).map(move|t|(r,t))).zip(before){let actual=gems(&s,&o,r,t).unwrap();assert_eq!(actual.len(),expected.len());for (a,b) in actual.iter().zip(expected){assert_eq!((a.tick,a.lane,a.length),(b.tick+5760,b.lane,b.length));}}
        let (shifted,report)=build_chart(&s,&o).unwrap();
        assert!(shifted.contains("7680 = E \"section Verse\""));assert_eq!(report["alignment"]["chart_shift_ticks"],5760);
        let phrases=|chart:&str|chart.lines().filter(|l|l.contains(" = S 2 ")).map(|l|{let (tick,event)=l.trim().split_once(" = ").unwrap();(tick.parse::<i64>().unwrap(),event.to_owned())}).collect::<Vec<_>>();
        assert_eq!(phrases(&shifted),phrases(&original).into_iter().map(|(t,e)|(t+5760,e)).collect::<Vec<_>>());
        o.chart_shift_bars=-1;assert!(build_chart(&s,&o).is_err());
        for track in &mut s.tracks{for note in &mut track.notes{note.tick+=3840.;}}s.end+=3840.;
        assert_eq!(minimum_chart_shift_bars(&s,&o),-2);o.chart_shift_bars=-2;assert_eq!(gems(&s,&o,0,3).unwrap()[0].tick,0);
    }
    #[test]
    fn old_projects_preserve_alignment_and_new_values_round_trip() {
        let mut saved=serde_json::to_value(Options{offset:250,..Default::default()}).unwrap();
        saved.as_object_mut().unwrap().remove("audio_shift_ms");saved.as_object_mut().unwrap().remove("chart_shift_bars");saved.as_object_mut().unwrap().remove("constant_bpm");saved.as_object_mut().unwrap().remove("lyric_text");saved.as_object_mut().unwrap().remove("lyric_repairs");
        let mut o:Options=serde_json::from_value(saved).unwrap();assert_eq!(o.audio_alignment_ms(),-250);assert_eq!(o.chart_shift_bars,0);assert!(!o.constant_bpm);assert!(o.lyric_text.is_none());
        o.set_audio_alignment_ms(400);o.chart_shift_bars=2;o.constant_bpm=true;
        let reloaded:Options=serde_json::from_slice(&serde_json::to_vec(&o).unwrap()).unwrap();assert_eq!(reloaded.audio_alignment_ms(),400);assert_eq!(reloaded.chart_shift_bars,2);assert!(reloaded.constant_bpm);
    }
    #[test]
    fn export_bakes_main_and_stem_alignment_without_double_delay() {
        let root=std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join(format!("chart-export-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));fs::create_dir_all(&root).unwrap();
        let input=root.join("recording.wav");let mut wav=hound::WavWriter::create(&input,hound::WavSpec{channels:1,sample_rate:8000,bits_per_sample:16,sample_format:hound::SampleFormat::Int}).unwrap();for _ in 0..16000{wav.write_sample(1200_i16).unwrap();}wav.finalize().unwrap();let original=fs::read(&input).unwrap();
        let s=score();let mut o=Options{title:"Alignment test".into(),tracks:[Some(0),None,None,None],output:root.clone(),audio:Some(input.clone()),lyrics:Some(asset_root().join("examples/demo.lrc")),chart_shift_bars:1,..Default::default()};
        o.extras.insert("Guitar stem".into(),input.clone());o.extras.insert("Preview audio".into(),input.clone());
        for shift in [250,-250]{o.set_audio_alignment_ms(shift);let exported=export(&s,&o).unwrap();let main=hound::WavReader::open(exported.join("song.wav")).unwrap();let stem=hound::WavReader::open(exported.join("guitar.wav")).unwrap();assert_eq!(main.duration(),(16000_i64+shift*8) as u32);assert_eq!(main.duration(),stem.duration());assert_eq!(main.spec().sample_rate,8000);assert_eq!(main.spec().channels,1);assert_eq!(main.spec().bits_per_sample,24);assert_eq!(fs::read(exported.join("preview.wav")).unwrap(),original);let ini=fs::read_to_string(exported.join("song.ini")).unwrap();assert!(ini.contains("delay = 0\n"));let chart=fs::read_to_string(exported.join("notes.chart")).unwrap();assert!(chart.contains("MusicStream = \"song.wav\""));assert!(chart.contains("1920 = N 0 0"));let report:serde_json::Value=serde_json::from_slice(&fs::read(exported.join("conversion-report.json")).unwrap()).unwrap();assert_eq!(report["alignment"]["audio_shift_ms"],shift);assert!(report["alignment"]["audio_baked"].as_bool().unwrap());assert!(report["adjusted_lrc"].as_str().unwrap().contains(if shift>0{"[00:02.250]With a beat"}else{"[00:01.750]With a beat"}));}
        assert_eq!(fs::read(&input).unwrap(),original);
        o.set_audio_alignment_ms(-3000);assert!(export(&s,&o).is_err());assert!(!fs::read_dir(&root).unwrap().any(|entry|entry.unwrap().file_name().to_string_lossy().starts_with(".chart-starter-")));
        let _=fs::remove_dir_all(root);
    }
    fn score()->Score{Score{ppq:480,end:7680.,tracks:vec![Track{index:0,name:"Drums".into(),percussion:true,notes:(0..32).map(|i|Note{tick:(i*120) as f64,pitch:36,velocity:100,..Default::default()}).collect()}],..Default::default()}}
    #[test]fn alternating_double_kick(){let s=score();let o=Options{bpm:140.,tracks:[Some(0),None,None,None],..Default::default()};let g=gems(&s,&o,0,3).unwrap();assert_eq!(g[0].lane,0);assert_eq!(g[1].lane,32);assert_eq!(g[2].lane,0);}
    #[test]fn reductions_are_less_dense(){let mut s=score();for (i,n) in s.tracks[0].notes.iter_mut().enumerate(){n.tick=(i*60) as f64;}let o=Options{tracks:[Some(0),None,None,None],..Default::default()};let counts:Vec<_>=(0..4).map(|d|gems(&s,&o,0,d).unwrap().len()).collect();assert!(counts.windows(2).all(|w|w[0]<w[1]));}
    #[test]fn lrc_multiple_timestamps_and_enhanced(){let r=parse_lrc("[offset:100]\n[00:01.50][00:03.000]Hello\n[00:04.00]<00:04.00>one <00:04.50>two").unwrap();assert_eq!(r.len(),4);assert_eq!(r[0].ms,1600);assert_eq!(r[3].ms,4600);}
    #[test]fn filename_safety(){assert_eq!(safe_name("CON"),"_CON");assert_eq!(safe_name("a/b:*"),"a_b__");}
    #[test]fn strict_collision_is_rejected(){let mut s=score();s.tracks[0].notes.push(Note{tick:0.,pitch:35,..Default::default()});let o=Options{tracks:[Some(0),None,None,None],..Default::default()};assert!(gems(&s,&o,0,3).is_err());}
    #[test]fn invalid_tempo_and_empty_selection_rejected(){let s=score();let mut o=Options::default();assert!(build_chart(&s,&o).is_err());o.tracks[0]=Some(0);o.bpm=f64::NAN;assert!(build_chart(&s,&o).is_err());}
    #[test]fn chart_events_are_chronological(){let s=score();let o=Options{tracks:[Some(0),None,None,None],..Default::default()};let (chart,_)=build_chart(&s,&o).unwrap();let mut previous=0;for line in chart.lines(){if line=="{"{previous=0;}if let Some((tick,_))=line.trim().split_once(" = "){if let Ok(tick)=tick.parse::<i64>(){assert!(tick>=previous);previous=tick;}}}}
    #[test]fn pitched_roles_emit_correct_sections(){let mut s=score();s.tracks[0].percussion=false;for (i,n) in s.tracks[0].notes.iter_mut().enumerate(){n.pitch=60+(i%8) as i32;n.length=480.;}let o=Options{tracks:[None,Some(0),Some(0),Some(0)],..Default::default()};let (chart,report)=build_chart(&s,&o).unwrap();for suffix in ["Single","DoubleBass","DoubleRhythm"]{for tier in TIERS{assert!(chart.contains(&format!("[{tier}{suffix}]")));}}assert_eq!(report["instruments"].as_array().unwrap().len(),3);}

    #[test]
    fn collisions_follow_reassignment_and_strict_export() {
        let mut s=score();
        s.tracks[0].notes=vec![38,40,42].into_iter().map(|pitch|Note{tick:480.,pitch,..Default::default()}).collect();
        let mut o=Options{tracks:[Some(0),None,None,None],..Default::default()};
        assert_eq!(drum_collisions(&s,&o)[0].pitches,vec![38,40]);
        assert!(gems(&s,&o,0,3).is_err());
        o.drum_map.insert(42,1);
        assert_eq!(drum_collisions(&s,&o)[0].pitches.len()-1,2);
        o.drum_map.insert(40,3);
        assert_eq!(drum_collisions(&s,&o)[0].pitches,vec![38,42]);
        o.drum_map.insert(42,66);
        assert!(drum_collisions(&s,&o).is_empty());
        assert!(gems(&s,&o,0,3).is_ok());
        o.drum_map.insert(40,1);
        o.drum_map.insert(40,-1);
        assert!(drum_collisions(&s,&o).is_empty());
    }

    #[test]
    fn cymbals_and_toms_collide_on_the_same_physical_lane() {
        let mut s=score();
        s.tracks[0].notes=vec![Note{tick:960.2,pitch:48,..Default::default()},Note{tick:960.4,pitch:42,..Default::default()}];
        let mut o=Options{tracks:[Some(0),None,None,None],..Default::default()};
        let collisions=drum_collisions(&s,&o);
        assert_eq!(collisions,vec![DrumCollision{tick:960,lane:2,pitches:vec![42,48]}]);
        o.strict=false;
        assert_eq!(drum_collisions(&s,&o),collisions);
        s.tracks[0].notes[1].tick=961.;
        assert!(drum_collisions(&s,&o).is_empty());
        s.tracks[0].notes[1].tick=960.;
        o.tracks[0]=None;
        assert!(drum_collisions(&s,&o).is_empty());
    }

    #[test]
    fn overall_bpm_changes_export_and_lyric_timing_without_moving_notes() {
        let s=score();
        let mut o=Options{bpm:90.,double_bass:false,tracks:[Some(0),None,None,None],lyrics:Some(asset_root().join("examples/demo.lrc")),..Default::default()};
        let original=gems(&s,&o,0,3).unwrap();
        let slow_rating=rating(&original,&s,&o);
        let (slow,_)=build_chart(&s,&o).unwrap();
        assert!(slow.contains("0 = B 90000"));
        assert!(slow.contains("1440 = E \"lyric With a beat\""));
        o.bpm=180.;
        let faster=gems(&s,&o,0,3).unwrap();
        assert_eq!(original.iter().map(|g|g.tick).collect::<Vec<_>>(),faster.iter().map(|g|g.tick).collect::<Vec<_>>());
        assert!(rating(&faster,&s,&o)>slow_rating);
        let (fast,report)=build_chart(&s,&o).unwrap();
        assert!(fast.contains("0 = B 180000"));
        assert!(fast.contains("2880 = E \"lyric With a beat\""));
        assert_eq!(report["bpm"],180.);
    }
    #[test]fn standard_phrases_extend_to_the_next_start_across_tempo_changes(){let mut s=score();s.tempos=vec![Tempo{tick:0.,bpm:120.},Tempo{tick:960.,bpm:240.}];let o=Options{tracks:[Some(0),None,None,None],lyric_text:Some("[00:01.000]First\n[00:06.000]Next\n[00:09.000]Last".into()),..Default::default()};let(chart,_)=build_chart(&s,&o).unwrap();let markers=chart.lines().filter(|l|l.contains("phrase_start")||l.contains("phrase_end")).map(|l|{let(t,v)=l.trim().split_once(" = ").unwrap();(t.parse::<i64>().unwrap(),v.to_owned())}).collect::<Vec<_>>();let starts:Vec<_>=markers.iter().filter(|(_,v)|v.contains("phrase_start")).map(|p|p.0).collect();let ends:Vec<_>=markers.iter().filter(|(_,v)|v.contains("phrase_end")).map(|p|p.0).collect();assert_eq!(starts,vec![960,10560,16320]);assert_eq!(ends[0],starts[1]-1);assert_eq!(ends[1],starts[2]-1);assert!(ends[2]>starts[2]);}
    #[test]fn enhanced_words_share_a_phrase_and_alignment_moves_only_lyrics(){let s=score();let mut o=Options{tracks:[Some(0),None,None,None],lyric_text:Some("[00:01.000]<00:01.000>A <00:00.500>B <00:03.000>C\n[00:05.000]Next".into()),..Default::default()};let before=gems(&s,&o,0,3).unwrap();let(c,r)=build_chart(&s,&o).unwrap();assert_eq!(c.matches("phrase_start").count(),2);assert!(c.contains("1920 = E \"lyric B\""));assert_eq!(r["lyric_repairs"][0]["repaired_ms"],2000);o.lyric_offset=1250;let(c,r)=build_chart(&s,&o).unwrap();assert!(c.contains("3120 = E \"lyric B\""));assert!(r["adjusted_lrc"].as_str().unwrap().contains("<00:03.250>B"));assert_eq!(gems(&s,&o,0,3).unwrap().iter().map(|n|n.tick).collect::<Vec<_>>(),before.iter().map(|n|n.tick).collect::<Vec<_>>());assert_eq!(o.audio_alignment_ms(),0);}
    #[test]fn repaired_export_has_review_notes_and_no_vocal_midi(){let s=score();let root=std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("lrc-repair-export");fs::create_dir_all(&root).unwrap();let mut o=Options{tracks:[Some(0),None,None,None],output:root.clone(),title:"Repair test".into(),lyric_text:Some("[00:01]<00:01>A <00:00>B <00:03>C".into()),..Default::default()};let original=o.lyric_text.clone();let p=export(&s,&o).unwrap();assert!(!p.join("notes.mid").exists());assert!(!fs::read_to_string(p.join("song.ini")).unwrap().contains("diff_vocals"));assert!(fs::read_to_string(p.join("LYRIC-REVIEW.txt")).unwrap().contains("00:00.000 -> 00:02.000"));assert!(fs::read_to_string(p.join("lyrics.lrc")).unwrap().contains("<00:02.000>B"));assert_eq!(o.lyric_text,original);let doc=crate::lyrics::load(&o).unwrap();o.lyric_repairs=doc.repairs.clone();o.lyric_text=Some(doc.lrc(0));let roundtrip:Options=serde_json::from_slice(&serde_json::to_vec(&o).unwrap()).unwrap();let (_,r)=build_chart(&s,&roundtrip).unwrap();assert_eq!(r["lyric_repairs"].as_array().unwrap().len(),1);let _=fs::remove_dir_all(root);}

}
