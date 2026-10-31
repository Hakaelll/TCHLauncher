use std::{fs,io::{Cursor,Read,Write},path::{Path,PathBuf},sync::Mutex};

static IO:Mutex<()>=Mutex::new(());
pub enum Kind{Content,Skin}
pub struct CachedImage{path:PathBuf,max_bytes:u64,max_side:u32}

fn directory(path:&Path)->Result<(),String>{
    let meta=fs::symlink_metadata(path).map_err(|error|error.to_string())?;
    if !meta.is_dir()||meta.file_type().is_symlink()||path.canonicalize().map_err(|error|error.to_string())?!=path{return Err("La carpeta de imágenes en caché no es segura".into());}Ok(())
}
fn file(path:&Path)->Result<Option<fs::Metadata>,String>{
    match fs::symlink_metadata(path){
        Ok(meta) if meta.is_file()&&!meta.file_type().is_symlink()=>Ok(Some(meta)),
        Ok(_)=>Err("La imagen en caché no es un archivo normal".into()),
        Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(None),
        Err(error)=>Err(error.to_string()),
    }
}
impl CachedImage{
    pub fn new(root:&Path,kind:Kind,key:&str)->Result<Self,String>{
        let root=root.canonicalize().map_err(|error|error.to_string())?;
        let (name,max_bytes,max_side,key_len)=match kind{Kind::Content=>("content-icons",100_000,96,64),Kind::Skin=>("skin-cache",16_384,8,32)};
        if key.len()!=key_len||!key.bytes().all(|byte|byte.is_ascii_hexdigit()){return Err("Identificador de imagen en caché no válido".into());}
        let base=root.join(name);
        match fs::create_dir(&base){Ok(())=>(),Err(error) if error.kind()==std::io::ErrorKind::AlreadyExists=>(),Err(error)=>return Err(error.to_string())}
        directory(&base)?;
        let path=base.join(format!("{key}.png"));file(&path)?;
        Ok(Self{path,max_bytes,max_side})
    }
    pub fn read(&self)->Result<Option<(fs::Metadata,Vec<u8>)>,String>{
        let _guard=IO.lock().map_err(|_|"No se pudo acceder a la caché de imágenes")?;
        directory(self.path.parent().unwrap())?;
        let Some(meta)=file(&self.path)? else{return Ok(None);};
        if meta.len()==0||meta.len()>self.max_bytes{return Ok(None);}
        let mut bytes=Vec::new();fs::File::open(&self.path).map_err(|error|error.to_string())?.take(self.max_bytes+1).read_to_end(&mut bytes).map_err(|error|error.to_string())?;
        if bytes.len() as u64>self.max_bytes{return Ok(None);}
        let mut reader=image::ImageReader::with_format(Cursor::new(&bytes),image::ImageFormat::Png);
        let mut limits=image::Limits::default();limits.max_image_width=Some(self.max_side);limits.max_image_height=Some(self.max_side);limits.max_alloc=Some(512_000);reader.limits(limits);
        if reader.decode().is_err(){return Ok(None);}
        Ok(Some((meta,bytes)))
    }
    pub fn write(&self,bytes:&[u8])->Result<(),String>{
        let _guard=IO.lock().map_err(|_|"No se pudo acceder a la caché de imágenes")?;
        directory(self.path.parent().unwrap())?;file(&self.path)?;
        if bytes.len() as u64>self.max_bytes{return Err("Imagen en caché demasiado grande".into());}
        let temporary=self.path.with_extension(format!("part-{}",uuid::Uuid::new_v4()));
        let mut output=fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|error|error.to_string())?;
        let result=output.write_all(bytes).and_then(|_|output.sync_all()).map_err(|error|error.to_string());drop(output);
        let result=result.and_then(|_|{directory(self.path.parent().unwrap())?;file(&self.path)?;crate::download::replace_file(&temporary,&self.path)});
        if result.is_err(){let _=fs::remove_file(temporary);}result
    }
}
