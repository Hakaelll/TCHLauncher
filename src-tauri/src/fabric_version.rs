//! Read-only predicate checks for Fabric metadata. Unknown syntax is not incompatibility.
//! Semantics reference: Fabric Loader VersionPredicateParser/VersionComparisonOperator.
use std::cmp::Ordering;

struct Numeric<'a> { components:Vec<i32>, prerelease:Option<&'a str> }
impl Numeric<'_> {fn component(&self,index:usize)->i32{*self.components.get(index).unwrap_or(&0)}}
// BMP Nd zero points from UnicodeData 15.0. Integer.parseInt uses UTF-16 char
// digits, so supplementary-plane decimal digits are intentionally not numeric.
const DECIMAL_STARTS:&[u32]=&[0x30,0x660,0x6f0,0x7c0,0x966,0x9e6,0xa66,0xae6,0xb66,0xbe6,0xc66,0xce6,0xd66,0xde6,0xe50,0xed0,0xf20,0x1040,0x1090,0x17e0,0x1810,0x1946,0x19d0,0x1a80,0x1a90,0x1b50,0x1bb0,0x1c40,0x1c50,0xa620,0xa8d0,0xa900,0xa9d0,0xa9f0,0xaa50,0xabf0,0xff10];
fn component(part:&str)->Option<i32>{
    if part.is_empty(){return None;}
    part.chars().try_fold(0i32,|value,c|{
        let point=c as u32;
        let digit=DECIMAL_STARTS.iter().find_map(|start|point.checked_sub(*start).filter(|offset|*offset<10))?;
        value.checked_mul(10)?.checked_add(digit as i32)
    })
}
fn runtime_sensitive_digits(value:&str)->bool{
    let core=value.split_once('+').map_or(value,|(core,_)|core).split('-').next().unwrap_or("");
    // These two BMP blocks became decimal digits after Java 8's Unicode 6.2.
    // The static audit does not establish the server JVM's Unicode version.
    core.chars().any(|c|matches!(c as u32,0xde6..=0xdef|0xa9f0..=0xa9f9))
}
fn numeric(value:&str)->Option<Numeric<'_>> {
    let value=value.split_once('+').map_or(value,|(core,_)|core);
    let(core,prerelease)=value.split_once('-').map_or((value,None),|(core,pre)|(core,Some(pre)));
    if prerelease.is_some_and(|pre|!pre.is_empty()&&pre.split('.').any(|part|part.is_empty()||!part.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'))){return None;}
    let components=core.split('.').map(component).collect::<Option<Vec<_>>>()?;
    Some(Numeric{components,prerelease})
}
fn compare(a:&Numeric<'_>,b:&Numeric<'_>)->Ordering {
    for index in 0..a.components.len().max(b.components.len()) {
        let order=a.component(index).cmp(&b.component(index));if order!=Ordering::Equal{return order;}
    }
    match(a.prerelease,b.prerelease){
        (None,None)=>Ordering::Equal,(None,Some(_))=>Ordering::Greater,(Some(_),None)=>Ordering::Less,
        (Some(a),Some(b))=>{
            let mut a=a.split('.').filter(|part|!part.is_empty());let mut b=b.split('.').filter(|part|!part.is_empty());
            loop{match(a.next(),b.next()){
                (None,None)=>return Ordering::Equal,(Some(_),None)=>return Ordering::Greater,(None,Some(_))=>return Ordering::Less,
                (Some(a),Some(b))=>{
                    let integer=|s:&str|s=="0"||(!s.starts_with('0')&&s.bytes().all(|b|b.is_ascii_digit()));
                    let order=match(integer(a),integer(b)){
                        (true,true)=>a.len().cmp(&b.len()).then_with(||a.cmp(b)),
                        (true,false)=>Ordering::Less,(false,true)=>Ordering::Greater,(false,false)=>a.cmp(b),
                    };if order!=Ordering::Equal{return order;}
                }
            }}
        }
    }
}
fn term(version:&str,term:&str)->Option<bool>{
    if term=="*"||term.is_empty(){return Some(true);}
    if term.contains(['|','&','(',')']){return None;}
    let(op,reference)=[">=","<=",">","<","=","~","^"].into_iter().find_map(|op|term.strip_prefix(op).map(|reference|(op,reference))).unwrap_or(("=",term));
    if reference.is_empty(){return None;}
    if runtime_sensitive_digits(version)||runtime_sensitive_digits(reference){return None;}
    let core=reference.split_once('+').map_or(reference,|(core,_)|core);
    let parts:Vec<_>=core.split('.').collect();
    if let Some(index)=parts.iter().position(|part|matches!(*part,"x"|"X"|"*")){
        if op!="="||index==0||parts[index..].iter().any(|part|!matches!(*part,"x"|"X"|"*")){return None;}
        let Some(actual)=numeric(version) else{return Some(false);};
        let prefix=parts[..index].join(".");let lower=numeric(&prefix)?;
        // A wildcard includes all prereleases of the matching component prefix.
        return Some((0..index).all(|i|actual.component(i)==lower.component(i)));
    }
    if core.contains([ '*'])||core.contains(".x")||core.contains(".X"){return None;}
    match(numeric(version),numeric(reference)){
        (Some(actual),Some(reference))=>{
            let order=compare(&actual,&reference);
            Some(match op{
                ">="=>order!=Ordering::Less,"<="=>order!=Ordering::Greater,">"=>order==Ordering::Greater,"<"=>order==Ordering::Less,"="=>order==Ordering::Equal,
                "~"=>order!=Ordering::Less&&actual.component(0)==reference.component(0)&&actual.component(1)==reference.component(1),
                "^"=>order!=Ordering::Less&&actual.component(0)==reference.component(0),_=>return None,
            })
        }
        // Fabric treats inclusive operators on nonsemantic versions as exact equality.
        (_,None) if matches!(op,">"|"<")=>None,
        _=>Some(matches!(op,">="|"<="|"="|"~"|"^")&&version==reference),
    }
}
fn predicate(version:&str,predicate:&str)->Option<bool>{
    if predicate.len()>256||predicate.chars().any(char::is_control){return None;}
    let mut unknown=false;let mut mismatch=false;
    for value in predicate.split(' '){match term(version,value){Some(false)=>mismatch=true,None=>unknown=true,_=>{}}}
    // A malformed expression must never become negative evidence through another term.
    if unknown{None}else{Some(!mismatch)}
}
pub fn matches(version:&str,alternatives:&[String])->Option<bool>{
    if alternatives.is_empty()||alternatives.len()>64||version.is_empty()||version.len()>256||version.chars().any(char::is_control){return None;}
    let mut unknown=false;
    for range in alternatives{match predicate(version,range){Some(true)=>return Some(true),None=>unknown=true,_=>{}}}
    if unknown{None}else{Some(false)}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn check(version:&str,range:&str)->Option<bool>{matches(version,&[range.into()])}
    #[test]
    fn fabric_ranges_use_padded_components_and_its_major_caret_semantics(){
        for(v,r,expected)in[("1.2","=1.2.0",true),("1.2.1","=1.2",false),("0.9","^0.2",true),("1.3","~1.2",false),("1.2.9","~1.2",true),("1.21.11",">=1.21 <1.22",true),("1.22",">=1.21 <1.22",false),("1.2.3.4","1.2.3.x",true),("1.2.4","1.2.3.x",false),("1.2-beta","1.2.x",true),("1.3-alpha","1.2.x",false),("1.2.0+build","1.2",true)]{assert_eq!(check(v,r),Some(expected),"{v} / {r}");}
        assert_eq!(matches("2.5",&["1.x".into(),">=2 <3".into()]),Some(true));
    }
    #[test]
    fn prerelease_and_nonsemantic_versions_follow_fabric_comparisons(){
        for(v,r,expected)in[("1.0-alpha.9","<1.0-alpha.10",true),("1.0-9999999999999999999","<1.0-10000000000000000000",true),("1.0-01","<1.0-1",false),("1.0-","<1.0-alpha",true),("1.0-alpha","<1.0",true),("snapshot",">=snapshot",true),("snapshot",">=other",false),("1.2","=custom",false)]{assert_eq!(check(v,r),Some(expected),"{v} / {r}");}
        assert_eq!(check("snapshot",">snapshot"),None);
    }
    #[test]
    fn unrecognized_syntax_never_certifies_an_incompatibility(){
        for range in [">=",">= 1.2",">=1.x","1.x.2","1.2 || 2.0",">=2 <","1.x-beta"]{assert_eq!(check("1.0",range),None,"{range}");}
        assert_eq!(matches("1.0",&[">=2".into(),">".into()]),None);
        assert_eq!(matches("1.0",&["*".into(),">".into()]),Some(true));
    }
    #[test]
    fn unicode_components_use_java_decimal_values_without_overflow_or_supplementary_digits(){
        assert_eq!(check("1.2",">=١.٢"),Some(true));assert_eq!(check("१.२","1.2.x"),Some(true));
        assert_eq!(check("１.２","=1.2"),Some(true));assert_eq!(check("１.２","<1.1"),Some(false));
        assert_eq!(component("٢١٤٧٤٨٣٦٤٧"),Some(i32::MAX));assert_eq!(component("٢١٤٧٤٨٣٦٤٨"),None);
        assert_eq!(component("𝟙"),None);assert_eq!(component("²"),None);assert_eq!(component("Ⅳ"),None);
        assert_eq!(check("෧.෨","=1.2"),None);assert_eq!(check("1.2",">=꧱.꧲"),None);
        assert_eq!(check("1.2+෧","=1.2"),Some(true));assert_eq!(check("෧.෨","*"),Some(true));
    }
    #[test]
    #[ignore = "Requires TCH_TEST_JAVA and TCH_TEST_FABRIC_CLASSPATH with the official loader and compiled FabricPredicateProbe; never runs Minecraft"]
    fn recognized_predicates_match_the_real_fabric_api(){
        use base64::Engine;
        let java=std::env::var_os("TCH_TEST_JAVA").expect("TCH_TEST_JAVA required");
        let classpath=std::env::var_os("TCH_TEST_FABRIC_CLASSPATH").expect("TCH_TEST_FABRIC_CLASSPATH required");
        let versions=["0","0.0.1","0.2","0.9","1","1.0","1.0.0","1.0.1","1.2","1.2.0.1","1.2.3.4","2.0","1.0-","1.0-alpha","1.0-alpha.9","1.0-alpha.10","1.0-01","1.0-1","1.0-9999999999999999999","1.0-10000000000000000000","1.0+build","1.0+","01.02","snapshot","custom","1.21.11","2147483648","1.0-alpha..1","١.٢","१.२","1.0+λ","snapshot-ñ"];
        let ranges=["*","","1","1.0","=1.0.0",">=1.0","<=1.0",">1.0","<1.0","^0.2","~1.0","^1.0","1.x","1.X","1.*","1.x.x","1.0.x","1.2.3.x",">=1 <2",">=1.0-alpha <1.0","<1.0-alpha.10",">1.0-01","<1.0-10000000000000000000","snapshot",">=snapshot","~snapshot",">snapshot","custom","2147483648","1.0+build","1.0+other","1.0-",">=1.21 <1.22",">=","1.x-beta","1.x.2",">=1.x","1.0 || 2.0",">= 1.0",">=١.٢","१.२"];
        let mut cases:Vec<(String,String)>=versions.iter().flat_map(|version|ranges.iter().map(move|range|(version.to_string(),range.to_string()))).collect();
        for start in DECIMAL_STARTS{
            let one=char::from_u32(start+1).unwrap();let two=char::from_u32(start+2).unwrap();
            let version=format!("{one}.{two}");
            for range in ["=1.2",">=1 <2","1.2.x","<1.2"]{cases.push((version.clone(),range.into()));}
            cases.push(("1.2".into(),format!(">={version}")));
        }
        struct Temp(std::path::PathBuf);impl Drop for Temp{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let root=Temp(std::env::temp_dir().join(format!("tch-fabric-predicate-{}",uuid::Uuid::new_v4())));std::fs::create_dir_all(&root.0).unwrap();
        let path=root.0.join("cases.tsv");let encoder=base64::engine::general_purpose::STANDARD;
        let input=cases.iter().map(|(version,range)|format!("{}\t{}\n",encoder.encode(version),encoder.encode(range))).collect::<String>();std::fs::write(&path,input).unwrap();
        let mut command=std::process::Command::new(java);
        command.args(["-Xms16m","-Xmx64m","-cp"]).arg(classpath).arg("FabricPredicateProbe").arg(&path).current_dir(&root.0).stdin(std::process::Stdio::null());
        #[cfg(windows)]{use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
        let output=command.output().unwrap();assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
        let output=String::from_utf8(output.stdout).unwrap();let outcomes:Vec<_>=output.lines().collect();assert_eq!(outcomes.len(),cases.len());
        let mut checked=0;
        for((version,range),actual)in cases.iter().zip(outcomes){
            if let Some(expected)=check(version,range){checked+=1;assert_eq!(actual,if expected{"true"}else{"false"},"version={version:?}; range={range:?}");}
        }
        assert!(checked>800);println!("{checked} recognized comparisons match the real Fabric API ({} fixture cases)",cases.len());
    }
}
