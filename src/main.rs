use clap::{Parser, ValueHint, value_parser};
use threadpool::ThreadPool;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{ path::Path, process::Stdio, sync::mpsc::channel, u64};

#[derive(Parser)]
#[command(version="0.8", about="JXLCONVERT Batch convert image files to JXL")]
struct Cli {
    // The pattern to look for
    #[arg(value_hint=ValueHint::DirPath, default_value=".")]
    path: std::path::PathBuf,
    #[arg(value_hint=ValueHint::DirPath, default_value="./JXL")]
    output_path: std::path::PathBuf,
    #[arg(long, short, default_value = "cjxl")]
    encoder: String,
    // The quality
    #[arg(long, short, default_value_t = 85, value_parser = value_parser!(u32).range(1..100), value_name = "0..100")]
    quality: u32,
}



fn main() {
    let pool = ThreadPool::new(15);
    let args = Cli::parse();
    let _ = std::fs::create_dir(&args.output_path);
    let _count = std::fs::read_dir(&args.path).unwrap_or_else(|error|{
        panic!("Problem reading the file {error:?}")
    }).count();
    let mut count_file: u64 = 0;
    let mut current_dir_file_only: Vec<std::fs::DirEntry> = Vec::new();
    let mut list_file: Vec<_> = std::fs::read_dir(&args.path).unwrap_or_else(|error|{panic!("Problem readint the file {error:?}")})
        .map(|r| r.expect("Can't map file {r:?} into array from path")).collect::<Vec<_>>();
    let file_types = ["jpg", "jpeg", "png", "webp", "tiff", "tif"];
    list_file.sort_by_key(|dir| dir.path());
    for n in list_file {
        let direntry = n;
        if &direntry.file_type().unwrap_or_else(|error|{panic!("Couldn't extract file type from {direntry:?} {error:?}")}).is_file() == &true {
            // if direntry.path().file_name().filter() {
            //     println!("file {:?} starts with a dot (.), ignoring", direntry.file_name());
            //     continue;
            // }
            let extension = &direntry.path().extension()
                .unwrap_or_else(||{panic!("Couldn't get the extension {direntry:?}")}).display().to_string().to_lowercase();
            if file_types.iter().any(|x| extension.contains(x)){
            current_dir_file_only.push(direntry);
            count_file += 1;
            }
        };     
    };
    println!("All entries {}, All Image Files: {}", &_count, current_dir_file_only.iter().count());
    if count_file == 0 {return;};
    if current_dir_file_only.iter().count() == 0 {return;};
    let multibar = MultiProgress::new();
    let sty = ProgressStyle::with_template("[{elapsed_precise}] {msg} {bar:30.cyan/blue} {pos}/{len}")
        .unwrap_or_else(|err|{panic!("Something wrong with the progress template {err:?}")})
        .progress_chars("##-");
    let bar = multibar.add(ProgressBar::new(count_file));
    let bar2 = multibar.add(ProgressBar::new(count_file));
    bar.set_style(sty.clone());
    bar2.set_style(sty.clone());
    let (tx_message, rx_message) = channel::<String>();
    let mut file_count: u64 = 0;

    for entry in current_dir_file_only {
        let mut encoder: String = args.encoder.clone();
        let output_dir = args.output_path.clone();
        file_count += 1;
        let tx_message = tx_message.clone();
        pool.execute(move|| {
            let quality_str = &args.quality.to_string();
            let file = entry;
            let image_file_display: &str = &file.path().display().to_string();
            let output_file = &file.path().with_extension("JXL");
            let output_file_only = &output_file.file_name().unwrap_or_else(||{panic!("Expected file name from {output_file:?}")});
            let mut final_file = std::path::PathBuf::new();
            final_file.push(&output_dir);
            final_file.push(&output_file_only);
            let final_file = final_file.to_str().unwrap_or_else(||{panic!("Couldn't convert {final_file:?} to str")});
            // println!("Converting: {} -> {}",&image_file.file_name().expect("file not found").display(), &output_file_only.display());
            let cjxl_file_types = ["jpg", "jpeg", "png", "gif"];
            let extension = &file.path().extension().expect("Couldn't Get Extension").display().to_string().to_lowercase();
            if !cjxl_file_types.iter().any(|x| extension.contains(x)) {
                encoder = String::from("vips");
                let message = format!("Converting Using VIPS for -> {}", &output_file_only.display());
                tx_message.send(message.clone()).expect(&message)
            }; 
            let already_exists = std::fs::exists(&final_file).unwrap_or_else(|e|{panic!("Couldn't check if file exists {e:?}")});
            if !already_exists {
                if encoder == "cjxl" {
                    // thread::sleep(time::Duration::from_millis(200));
                    let _convert = std::process::Command::new("cjxl")
                        .args([&image_file_display, &final_file,"--effort=7", "-q", &quality_str, "--lossless_jpeg=0", "--quiet"])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .unwrap_or_else(|e|{panic!("Couldn't spawn cjxl for {image_file_display:?} {e:?}")});
                    // println!("Convert CJXL message for {image_file_display:?} {_convert:?}");

                    let final_path = Path::new(&final_file);
                    let converted_file = match std::fs::exists(&final_path) {
                        Ok(true) => true,
                        Ok(false) => false,
                        Err(e) => panic!("fuck {}", e),
                    };

                    
                    if !converted_file {
                        encoder = String::from("vips");
                        eprintln!("Couldn't find output file, using VIPS to convert {:?}", &image_file_display);
                        // return;
                    } else if converted_file {
                        // return;
                        // eprintln!("Could find output file{:?}", &image_file_display)
                        let message = format!("Converting: {} -> {}",&file.path().file_name()
                            .unwrap_or_else(||{panic!("File {file:?} not found")}).display(), &output_file_only.display());
                        let message_error = format!("ERROR sending convert message: {} -> {}",&file.path().file_name()
                            .unwrap_or_else(||{panic!("File {file:?} not found")}).display(), &output_file_only.display());
                        tx_message.send(message.clone()).expect(&message_error);

                        // thread::sleep(time::Duration::from_millis(200));
                        let _exiftool = std::process::Command::new("exiftool")
                            .args(["-m", "-TagsFromFile", &image_file_display, "-all", "-FileModifyDate<FileModifyDate", &final_file, "-overwrite_original"])
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status()
                            .unwrap_or_else(|e|{panic!("Couldn't read status on Exiftool for file {image_file_display:?} {e:?}")});
                        // println!("Convert EXIF message {_exiftool:?}");
                        let convert_message = format!("Copying Metadata for {}", &output_file_only.display());
                        let convert_message_error = format!("ERROR sending exif message {}", &output_file_only.display());
                        tx_message.send(convert_message).expect(&convert_message_error);
                    };
                }

                let final_path = Path::new(&final_file);
                let converted_file = std::fs::exists(&final_path); 
                
                if converted_file.is_err() {
                    encoder = String::from("vips");
                    eprintln!("Couldn't find output file, using VIPS to convert {:?}", &image_file_display)
                } else if converted_file.is_ok() {
                    // return;
                    // eprintln!("Could find output file{:?}", &image_file_display)
                };

                if encoder == "vips" {
                    let message = format!("Converting: {} -> {}",&file.path().file_name()
                        .expect("File not found").display(), &output_file_only.display());
                    let message_error = format!("ERROR sending convert message: {} -> {}",&file.path().file_name()
                        .expect("File {file:?} not found").display(), &output_file_only.display());
                    tx_message.send(message.clone()).expect(&message_error);
                    let _convert = std::process::Command::new("vips")
                        .args(["jxlsave", &image_file_display, &final_file, "-Q", &quality_str])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .unwrap_or_else(|e|{panic!("Couldn't spawn vips for {image_file_display:?} {e:?}")});
                    // println!("Convert VIPS message for {image_file_display:?} {_convert:?}");

                    let convert_message = format!("Copying Metadata for {}", &output_file_only.display());
                    let convert_message_error = format!("ERROR sending exif message for {}", &output_file_only.display());
                    tx_message.send(convert_message).expect(&convert_message_error);
                    let _exiftool = std::process::Command::new("exiftool")
                        .args(["-m","-IPTCDigest=new", "-TagsFromFile", &image_file_display, "-all", "-FileModifyDate<FileModifyDate", &final_file, "-overwrite_original"])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .unwrap_or_else(|e|{panic!("Couldn't read status on Exiftool for file {image_file_display:?} {e:?}")});
                    // println!("Convert EXIF message {_exiftool:?}");

                } 

                if encoder != "vips" && encoder != "cjxl" {
                    panic!("Wrong convert type for {:?}, please use vips or cjxl as the encoder, encoder = {}", &image_file_display, &encoder);
                };
                if file_count == count_file {
                    let done  = String::from("done");
                    tx_message.send(done).expect("Couldn't send end channel message");
                    return;
                };
                return;
            } else {
                // eprintln!("File {:?} already exists, skipping", &output_file_only);
                let _ = tx_message.send(String::from("File already exists"));
                return;
            };
        });
        if file_count == count_file {
            break;
        };
    };
    loop {
        // let panic = pool.panic_count();
        // let active = pool.active_count();
        // let queued = pool.queued_count();
        // println!("Recv: {}, Panic: {}, Active: {}, Queued: {}",&recv , panic, active, queued);
        let recv = rx_message.recv().unwrap_or_else(|e|{panic!("Couldn't recieve message {e:?}")});
        if recv == String::from("done") {
            bar.finish_with_message("All files converted to JXL");
            bar2.finish_with_message("All metadata transferred to JXL");
            println!("We're Done");
            drop(tx_message);
            break;
        };
        if recv.starts_with("Converting") {
            if recv.contains("VIPS") {
                // eprintln!("{:?}", &recv);
            }
            bar.set_message(recv);
            bar.inc(1);
            continue;
        } else if recv.starts_with("Copying") {
            bar2.set_message(recv);
            bar2.inc(1);
            continue;
        } else if recv.starts_with("File") {
            bar.set_message(recv.clone());
            bar.inc(1);
            bar2.set_message(recv);
            bar2.inc(1);
            continue;
        }
        ;
    };
    pool.join();
}
