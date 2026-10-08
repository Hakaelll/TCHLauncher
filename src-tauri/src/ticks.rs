use serde::Serialize;
use std::time::{Duration,Instant};

const RESPONSE_TIMEOUT:Duration=Duration::from_secs(10);
const FRESH_FOR:Duration=Duration::from_secs(60);
const QUERY_INTERVAL:Duration=Duration::from_secs(15);

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Reading{pub one_minute:f64,pub five_minutes:f64,pub fifteen_minutes:f64,pub sampled_at:i64}

#[derive(Serialize)]
pub struct Snapshot{pub reading:Option<Reading>,pub pending:bool}

#[derive(Default)]
pub struct Collector{requested:Option<Instant>,last_request:Option<Instant>,reading:Option<(Instant,Reading)>}

impl Collector{
    pub fn begin(&mut self,now:Instant)->Result<(),String>{
        if self.last_request.is_some_and(|last|now.saturating_duration_since(last)<QUERY_INTERVAL){return Err("Espera 15 segundos entre consultas de TPS".into());}
        self.requested=Some(now);self.last_request=Some(now);Ok(())
    }
    pub fn failed(&mut self){self.requested=None;}
    pub fn receive(&mut self,line:&str,now:Instant,sampled_at:i64){
        let Some(requested)=self.requested else{return;};
        if now.saturating_duration_since(requested)>RESPONSE_TIMEOUT{self.requested=None;return;}
        if let Some(values)=parse(line){
            self.reading=Some((now,Reading{one_minute:values[0],five_minutes:values[1],fifteen_minutes:values[2],sampled_at}));self.requested=None;
        }
    }
    pub fn snapshot(&self,now:Instant)->Snapshot{
        Snapshot{reading:self.reading.as_ref().filter(|(at,_)|now.saturating_duration_since(*at)<=FRESH_FOR).map(|(_,value)|value.clone()),pending:self.requested.is_some_and(|at|now.saturating_duration_since(at)<=RESPONSE_TIMEOUT)}
    }
}

fn parse(line:&str)->Option<[f64;3]>{
    let clean=crate::log::without_terminal_colors(line);
    let clean=crate::log::without_terminal_prompt(&clean);
    let (prefix,message)=clean.split_once("]: ")?;
    // Accept only the two native INFO formatters, never chat or plugin prefixes.
    let time=prefix.as_bytes().get(..9)?;
    if time[0]!=b'['||time[3]!=b':'||time[6]!=b':'||[1,2,4,5,7,8].iter().any(|&index|!time[index].is_ascii_digit()){return None;}
    if &prefix[9..]!=" INFO"&&&prefix[9..]!="] [Server thread/INFO"{return None;}
    let mut values=message.strip_prefix("TPS from last 1m, 5m, 15m: ")?.split(',').map(|value|{
        let value=value.trim().strip_prefix('*').unwrap_or(value.trim());
        if value.is_empty()||!value.chars().all(|c|c.is_ascii_digit()||c=='.'){return None;}
        value.parse::<f64>().ok().filter(|number|number.is_finite()&&*number>=0.0&&*number<=100.0)
    });
    let reading=[values.next()??,values.next()??,values.next()??];
    values.next().is_none().then_some(reading)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn accepts_only_complete_native_tps_responses(){
        assert_eq!(parse("[12:01:02 INFO]: TPS from last 1m, 5m, 15m: 20.0, 19.5, 18.1"),Some([20.0,19.5,18.1]));
        assert_eq!(parse("\u{1b}[33m[12:01:02] [Server thread/INFO]: TPS from last 1m, 5m, 15m: *20.0, 19.5, 18.1\u{1b}[0m"),Some([20.0,19.5,18.1]));
        assert_eq!(parse("> > \r  \r[01:55:36 INFO]: \u{1b}[33mTPS from last 1m, 5m, 15m: \u{1b}[0m\u{1b}[92m20.0\u{1b}[0m, 20.0, 20.0"),Some([20.0;3]));
        for line in ["> > \r  \r[12:01:02 INFO]: <Alex> TPS from last 1m, 5m, 15m: 20,20,20","> > \r  \r[12:01:02 INFO]: [Plugin] TPS from last 1m, 5m, 15m: 20,20,20","TPS from last 1m, 5m, 15m: 20,20,20","[12:01:02 INFO]: <Alex> TPS from last 1m, 5m, 15m: 20,20,20","[12:01:02 INFO]: [Plugin] TPS from last 1m, 5m, 15m: 20,20,20","[12:01:02] [Worker/INFO]: TPS from last 1m, 5m, 15m: 20,20,20","[12:01:02 WARN]: TPS from last 1m, 5m, 15m: 20,20,20","[12:01:02 INFO]: TPS from last 1m, 5m, 15m: NaN,20,20","[12:01:02 INFO]: TPS from last 1m, 5m, 15m: -1,20,20","[12:01:02 INFO]: TPS from last 1m, 5m, 15m: 20,20","[12:01:02 INFO]: TPS from last 1m, 5m, 15m: 20,20,20,20","[12:01:02 INFO]: TPS from last 1m, 5m, 15m: 20,20,20 trailing"]{assert!(parse(line).is_none(),"{line}");}
    }
    #[test]
    fn requires_pending_queries_bounds_freshness_and_resets_with_each_process(){
        let now=Instant::now();let line="[12:01:02 INFO]: TPS from last 1m, 5m, 15m: 20,19,18";
        let mut collector=Collector::default();collector.receive(line,now,100);assert!(collector.snapshot(now).reading.is_none());
        collector.begin(now).unwrap();assert!(collector.snapshot(now).pending);assert!(collector.begin(now+Duration::from_secs(1)).is_err());
        collector.receive(line,now+Duration::from_secs(2),200);let snapshot=collector.snapshot(now+Duration::from_secs(3));assert!(!snapshot.pending);assert_eq!(snapshot.reading.unwrap().sampled_at,200);
        assert!(collector.snapshot(now+Duration::from_secs(63)).reading.is_none());
        collector.begin(now+Duration::from_secs(70)).unwrap();collector.receive(line,now+Duration::from_secs(81),300);assert!(!collector.snapshot(now+Duration::from_secs(81)).pending);assert!(collector.snapshot(now+Duration::from_secs(81)).reading.is_none());
        collector.begin(now+Duration::from_secs(85)).unwrap();collector.failed();assert!(!collector.snapshot(now+Duration::from_secs(85)).pending);
        assert!(Collector::default().snapshot(now).reading.is_none());
    }
}
