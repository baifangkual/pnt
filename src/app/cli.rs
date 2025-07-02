use crate::app::runtime::PntRuntimeContext;

pub mod args;

/// cli 运行 模式
pub fn cli_run(pnt: PntRuntimeContext) -> anyhow::Result<()> {
    if let Some(f) = pnt.cli_args.find {
        // to do find Impl
        let vec = pnt.storage.select_entry_by_name_like(&f);
        vec.into_iter()
            .enumerate()
            .for_each(|(i, entry)| println!("{:3}: {}", i + 1, entry.name));
    }
    Ok(())
}
