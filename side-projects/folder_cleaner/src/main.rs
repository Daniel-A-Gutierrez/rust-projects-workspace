//cli tool to walk a directory and delete all the files that match a criteria
//works on filename (regex), file size, and extension (exact match)
//in retrospect i couldve done this easily in bash...
#![allow(unused_parens)]
#![feature(test)]
mod args;
use anyhow::Result;
use args::*;
use clap::Parser;
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::stdin;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

#[cfg(test)]
mod test
{
    extern crate test;
    use std::str::FromStr;

    use super::*;
    #[test]
    fn delete_extensions() -> Result<()>
    {
        let cli = Args { action:       Action::Delete,
                         target:       PathBuf::from_str("./Steins;Gate/").unwrap(),
                         matching:     None,
                         extension:    Some("m3u8".to_string()),
                         smaller_than: None,
                         larger_than:  None, };
        clean_folder(cli)
    }
}

fn main() -> Result<()>
{
    let cli = Args::parse();
    clean_folder(cli)
}

#[rustfmt::skip]
fn clean_folder(cli : Args) -> Result<()>
{
    let mut filters = vec![];
    add_lrg_filter(cli.larger_than, &mut filters);
    add_extension_filter(cli.extension, &mut filters);
    add_filename_filter(cli.matching, &mut filters);
    add_sm_filter(cli.larger_than, &mut filters);
    let contents = WalkDir::new(&cli.target);
    let matches: Vec<_> = contents.into_iter().skip(1)
                                  .filter_map(|entry| {
                                      let entry = entry.unwrap();
                                      let path = entry.path();
                                      let is_match = filters.iter().map(|f| f(&entry)).all(|b| b);
                                      if is_match {Some((path.to_owned()))}
                                      else {None}
                                  })
                                  .collect();
    match &cli.action
    {
        Action::Delete {} => println!("The following files and directories will be deleted!"),
        Action::Move { dst } =>
        {
            if (dst.exists())
            {
                assert!(dst.is_dir(), "Specified destination is not a directory");
            }
            else
            {
                std::fs::create_dir_all(dst)?;
            }
            println!("The following files will be moved to {:?}.", dst);
        }
    }

    for (path) in &matches
    {
        println!("{:?}", path);
    }
    println!("\nConfirm?[y/n]");
    let mut response = String::new();
    loop
    {
        response.clear();
        stdin().read_line(&mut response)?;
        match response.to_lowercase().trim()
        {
            "n" | "no" =>
            {
                println!("Aborting.");
                return Ok(());
            }
            "y" | "yes" =>
            {
                println!("Continuing.");
                break;
            }
            _ =>
            {
                println!("Respond with either 'yes', 'no', 'y', or 'n'.")
            }
        }
    }
    match cli.action
    {
        Action::Delete {} =>
        {
            for (path) in matches.into_iter()
            {
                //everythings either a dir or a file, no symlinks.
                let res = match &path.is_dir()
                {
                    true => std::fs::remove_dir_all(&path),
                    false => std::fs::remove_file(&path),
                };
                match res
                {
                    Ok(_) => {}
                    Err(e) => println!("Error deleting {:?}, {:?}", path, e),
                }
            }
        }
        Action::Move { dst } =>
        {
            for path in matches.into_iter()
            {
                let mut dclone = dst.clone();
                dclone.push(path.strip_prefix(&cli.target).unwrap().to_path_buf());
                match std::fs::rename(&path, dclone)
                {
                    Ok(_) =>
                    {}
                    Err(e) =>
                    {
                        println!("Error moving item: {:?}, {:?}", path, e)
                    }
                }
            }
        }
    }
    println!("Done");
    return Ok(());
}

fn add_sm_filter(size: Option<usize>, filters: &mut Vec<Box<dyn Fn(&DirEntry) -> bool>>)
{
    if let Some(x) =
        size.map(|s| {
                let closure: Box<dyn Fn(&DirEntry) -> bool> =
                    Box::new(move |d: &DirEntry| d.metadata().unwrap().len() < s as u64);
                return closure;
            })
    {
        filters.push(x);
    }
}

fn add_lrg_filter(size: Option<usize>, filters: &mut Vec<Box<dyn Fn(&DirEntry) -> bool>>)
{
    if let Some(x) =
        size.map(|s| {
                let closure: Box<dyn Fn(&DirEntry) -> bool> =
                    Box::new(move |d: &DirEntry| d.metadata().unwrap().len() > s as u64);
                return closure;
            })
    {
        filters.push(x);
    }
}

fn add_extension_filter(ext: Option<String>, filters: &mut Vec<Box<dyn Fn(&DirEntry) -> bool>>)
{
    if let Some(x) =
        ext.map(|ext| {
               let closure: Box<dyn Fn(&DirEntry) -> bool> =
                   Box::new(move |d: &DirEntry| d.path().extension().is_some_and(|e| *e == *ext));
               return closure;
           })
    {
        filters.push(x);
    }
}

fn add_filename_filter(rgx: Option<String>, filters: &mut Vec<Box<dyn Fn(&DirEntry) -> bool>>)
{
    if let Some(x) = rgx.map(|rgx| {
                            let rgx = Regex::new(&rgx).expect("Invalid regex");
                            let closure: Box<dyn Fn(&DirEntry) -> bool> =
                                Box::new(move |d: &DirEntry| {
                                    d.path()
                                     .file_name()
                                     .is_some_and(|fname| rgx.is_match(&fname.to_string_lossy()))
                                });
                            return closure;
                        })
    {
        filters.push(x);
    }
}
