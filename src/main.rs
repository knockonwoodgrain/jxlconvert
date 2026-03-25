use clap::Parser;
use threadpool::ThreadPool;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use core::time;
use std::{ thread, sync::mpsc::channel, u64};

#[derive(Parser)]
struct Cli {
    // The pattern to look for
    path: std::path::PathBuf,
    output_path: std::path::PathBuf,
    #[arg(long, short, default_value = "cjxl")]
    encoder: String,
    // The quality
    #[arg(long, short, default_value_t = 85)]
    quality: u32,
}



fn main() {
    let pool = ThreadPool::new(15);
    let args = Cli::parse();
    let _ = std::fs::create_dir(&args.output_path);
    let count = std::fs::read_dir(&args.path).unwrap().count();
    println!("{}", count);
    let mut count_file: u64 = 0;
    let mut current_dir_file_only: Vec<std::fs::DirEntry> = Vec::new();
    let mut list_file: Vec<_> = std::fs::read_dir(&args.path).unwrap().map(|r| r.unwrap()).collect::<Vec<_>>();
    let file_types = ["jpg", "jpeg", "png", "webp", "tiff"];
    list_file.sort_by_key(|dir| dir.path());
    for n in list_file {
        let direntry = n;
        if &direntry.file_type().unwrap().is_file() == &true {
            let extension = &direntry.path().extension().unwrap().display().to_string().to_lowercase();
            if file_types.iter().any(|x| extension.contains(x)){
            current_dir_file_only.push(direntry);
            count_file += 1;
            }
        };     
    };
    println!("All files only {}, current_dir_file_only count: {}", &count_file, current_dir_file_only.iter().count());
    let multibar = MultiProgress::new();
    let sty = ProgressStyle::with_template("[{elapsed_precise}] {msg} {bar:30.cyan/blue} {pos}/{len}").unwrap().progress_chars("##-");
    let bar = multibar.add(ProgressBar::new(count_file));
    let bar2 = multibar.add(ProgressBar::new(count_file));
    bar.set_style(sty.clone());
    bar2.set_style(sty.clone());
    let (tx_message, rx_message) = channel::<String>();
    let mut file_count: u64 = 0;

    for entry in current_dir_file_only {
        let encoder: String = args.encoder.clone();
        let output_dir = args.output_path.clone();
        file_count += 1;
        let tx_message = tx_message.clone();
        pool.execute(move|| {
            let quality_str = &args.quality.to_string();
            let file = entry;
            let image_file_display: &str = &file.path().display().to_string();
            let output_file = &file.path().with_extension("JXL");
            let output_file_only = &output_file.file_name().unwrap();
            let mut final_file = std::path::PathBuf::new();
            final_file.push(&output_dir);
            final_file.push(&output_file_only);
            let final_file = final_file.to_str().unwrap();
            // println!("Converting: {} -> {}",&image_file.file_name().expect("file not found").display(), &output_file_only.display());
            let already_exists = std::fs::exists(&final_file).unwrap();
            if !already_exists {
                if encoder == "cjxl" {
                    let message = format!("Converting: {} -> {}",&file.path().file_name().expect("file not found").display(), &output_file_only.display());
                    let message_error = format!("ERROR IN Converting: {} -> {}",&file.path().file_name().expect("file not found").display(), &output_file_only.display());
                    tx_message.send(message.clone()).expect(&message_error);
                    // thread::sleep(time::Duration::from_millis(200));
                    let _convert = std::process::Command::new("cjxl")
                        .args([&image_file_display, &final_file,"--effort=1", "-q", &quality_str, "--lossless_jpeg=0"])
                        .spawn()
                        .unwrap()
                        .wait();
                
                    let convert_message = format!("Copying Metadata for {}", &output_file_only.display());
                    let convert_message_error = format!("ERROR Copying Metadata for {}", &output_file_only.display());
                    tx_message.send(convert_message).expect(&convert_message_error);
                    // thread::sleep(time::Duration::from_millis(200));
                    let _exiftool = std::process::Command::new("exiftool")
                        .args(["-m", "-TagsFromFile", &image_file_display, "-UserComment", &final_file, "-overwrite_original"])
                        .stdout(std::process::Stdio::null()).status().unwrap();

                } else if encoder == "vips" {
                    let message = format!("Converting: {} -> {}",&file.path().file_name().expect("file not found").display(), &output_file_only.display());
                    let message_error = format!("ERROR IN Converting: {} -> {}",&file.path().file_name().expect("file not found").display(), &output_file_only.display());
                    tx_message.send(message.clone()).expect(&message_error);
                    let _convert = std::process::Command::new("vips").
                        args(["jxlsave", &image_file_display, &final_file, "-Q", &quality_str])
                        .spawn()
                        .unwrap()
                        .wait();

                    let convert_message = format!("Copying Metadata for {}", &output_file_only.display());
                    let convert_message_error = format!("ERROR Copying Metadata for {}", &output_file_only.display());
                    tx_message.send(convert_message).expect(&convert_message_error);
                    let _exiftool = std::process::Command::new("exiftool")
                        .args(["-m", "-TagsFromFile", &image_file_display, "-UserComment", &final_file, "-overwrite_original"])
                        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().unwrap();

                } else {
                    println!("Wrong convert type, ending program");
                    return;
                };
                if file_count == count_file {
                    let done  = String::from("done");
                    tx_message.send(done).expect("end channel");
                };
                return;
            } else {
                println!("File {:?} already exists", &output_file_only);
                return;
            };
        });
        if file_count == count_file {
            break;
        };
    };
    for recv in rx_message {
        // let panic = pool.panic_count();
        // let active = pool.active_count();
        // let queued = pool.queued_count();
        // println!("Recv: {}, Panic: {}, Active: {}, Queued: {}",&recv , panic, active, queued);
        if recv == String::from("done") {
            bar.finish_with_message("All files converted to JXL");
            bar2.finish_with_message("All files converted to JXL");
            println!("We're Done");
            drop(tx_message);
            break;
        }
        if recv.starts_with("Converting") {
            bar.set_message(recv);
            bar.inc(1);
        } else if recv.starts_with("Copying") {
            bar2.set_message(recv);
            bar2.inc(1);
        }
    }
    pool.join();
}
