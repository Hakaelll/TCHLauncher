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
