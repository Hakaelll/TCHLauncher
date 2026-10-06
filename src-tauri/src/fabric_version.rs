//! Read-only predicate checks for Fabric metadata. Unknown syntax is not incompatibility.
//! Semantics reference: Fabric Loader VersionPredicateParser/VersionComparisonOperator.
use std::cmp::Ordering;

struct Numeric<'a> { components:Vec<i32>, prerelease:Option<&'a str> }
impl Numeric<'_> {fn component(&self,index:usize)->i32{*self.components.get(index).unwrap_or(&0)}}
fn numeric(value:&str)->Option<Numeric<'_>> {
    let value=value.split_once('+').map_or(value,|(core,_)|core);
    let(core,prerelease)=value.split_once('-').map_or((value,None),|(core,pre)|(core,Some(pre)));
    if prerelease.is_some_and(|pre|!pre.is_empty()&&pre.split('.').any(|part|part.is_empty()||!part.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'))){return None;}
    let components=core.split('.').map(|part|{
        if part.is_empty()||!part.bytes().all(|b|b.is_ascii_digit()){return None;}
        part.parse::<i32>().ok()
    }).collect::<Option<Vec<_>>>()?;
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
}
