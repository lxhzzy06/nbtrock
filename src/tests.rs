#[allow(unused_imports)]
use crate::{IResult, NBT};

#[test]
fn read_example() -> IResult<()> {
    use std::fs::File;
    let mut f = File::open("res/download.nbt")?;
    let n = NBT::from_reader(&mut f)?;
    println!("{n}");
    Ok(())
}

#[test]
fn write_gold_farm_with_header() -> IResult<()> {
    use std::fs::File;
    use std::io::Write;
    let mut buf: Vec<u8> = vec![];
    let mut f = std::fs::File::create("res/out.nbt")?;
    NBT::from_reader(&mut File::open("res/example.nbt")?)?.write(&mut buf, true)?;
    f.write_all(&mut buf)?;
    Ok(())
}
