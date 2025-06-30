use std::error::Error;
use clap::Parser;

mod json_edit_exp;

fn main() -> Result<(), Box<dyn Error>> {

    
    
    
    
    
    let args = pnt::PntCmdLineArgs::parse();
    
    
    
    
    
    json_edit_exp::run()?;
    Ok(())
}
