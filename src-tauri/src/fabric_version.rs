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
