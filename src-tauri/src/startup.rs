use crate::model::LogEvent;
use serde::Serialize;

#[derive(Clone,Copy,Default,PartialEq,Eq,PartialOrd,Ord,Debug,Serialize)]
#[serde(rename_all="snake_case")]
pub enum Phase{#[default]Launching,LoadingWorld,PreparingSpawn,Ready}

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Event{pub server_id:String,pub session_id:String,pub phase:Phase}

impl Phase{
    pub fn observe(&mut self,event:&LogEvent){
        let next=match event.event.as_deref(){
            Some("world_load")=>Self::LoadingWorld,
            Some("spawn_prepare")=>Self::PreparingSpawn,
            Some("startup")=>Self::Ready,
            _=>return,
        };
        if next>*self{*self=next;}
    }
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn follows_observed_phases_without_guessing_or_regressing(){
        let mut phase=Phase::default();assert_eq!(phase,Phase::Launching);
        for (message,wanted) in [
            ("Loading properties",Phase::Launching),
            ("Preparing level \"world\"",Phase::LoadingWorld),
            ("Preparing start region for dimension minecraft:overworld",Phase::PreparingSpawn),
            ("Preparing level \"another\"",Phase::PreparingSpawn),
            ("Preparing spawn area: 100%",Phase::PreparingSpawn),
            ("Done (2.992s)! For help, type \"help\"",Phase::Ready),
            ("Preparing start region for dimension minecraft:the_end",Phase::Ready),
        ]{
            phase.observe(&crate::log::parse("id",&format!("[02:26:03 INFO]: {message}")));assert_eq!(phase,wanted);
        }
        let mut next=Phase::default();next.observe(&crate::log::parse("id","[02:26:04 INFO]: Done (0.1s)!"));assert_eq!(next,Phase::Ready);
        assert_eq!(Phase::default(),Phase::Launching);
    }
    #[test]fn player_plugin_and_invalid_progress_cannot_advance_startup(){
        let mut phase=Phase::default();
        for line in [
            "[02:26:03 INFO]: <Alex> Preparing level \"world\"",
            "[02:26:03 INFO]: Alex issued server command: /say Preparing spawn area: 100%",
            "[02:26:03 INFO]: [Plugin] Preparing level \"world\"",
            "[02:26:03 WARN]: Preparing level \"world\"",
            "[02:26:03] [Worker/INFO]: Preparing level \"world\"",
            "[02:26:03] [Worker/INFO]: Preparing level \"[Server thread/INFO]\"",
            "[02:26:03 INFO]: Preparing spawn area: 101%",
            "[02:26:03 INFO]: Preparing spawn area: NaN%",
            "[02:26:03 INFO]: Preparing spawn area: -1%",
            "[02:26:03 INFO]: Preparing spawn area: 42% still working",
        ]{phase.observe(&crate::log::parse("id",line));assert_eq!(phase,Phase::Launching,"{line}");}
        phase.observe(&crate::log::parse("id","\u{1b}[32m[02:26:03] [Server thread/INFO]: Preparing spawn area: 42%\u{1b}[0m"));assert_eq!(phase,Phase::PreparingSpawn);
    }
    #[test]
    #[ignore="Requires TCH_TEST_STARTUP_LOG pointing to an existing Minecraft console log; reads only"]
    fn recognizes_world_and_ready_in_an_actual_server_log(){
        let path=std::env::var("TCH_TEST_STARTUP_LOG").expect("Set TCH_TEST_STARTUP_LOG");
        let text=std::fs::read_to_string(path).unwrap();let mut phase=Phase::default();let mut loading=false;let mut spawning=false;
        for line in text.lines(){phase.observe(&crate::log::parse("fixture",line));loading|=phase==Phase::LoadingWorld;spawning|=phase==Phase::PreparingSpawn;}
        assert!(loading,"No recognized world-loading event");assert!(spawning,"No recognized spawn-preparation event");assert_eq!(phase,Phase::Ready);
    }
}
