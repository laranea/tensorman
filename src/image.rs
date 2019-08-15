use nix::unistd::geteuid;
use std::{
    env,
    fmt::{self, Display, Write},
    io,
    process::Command,
};

bitflags::bitflags! {
    pub struct TagVariants: u8 {
        const GPU = 1 << 0;
        const PY3 = 1 << 1;
        const JUPYTER = 1 << 2;
    }
}

/// A description of a Tensorflow Docker image, identified by its tag and tag variants.
pub struct Image<'a> {
    pub tag:      &'a str,
    pub variants: TagVariants,
}

impl<'a> Image<'a> {
    pub fn pull(&self) -> io::Result<()> {
        let mut command = Command::new("docker");
        command.args(&["pull", &String::from(self)]);
        eprintln!("{:?}", command);
        command.status().map(|_| ())
    }

    pub fn run(&self, cmd: &str, args: Option<&[&str]>) -> io::Result<()> {
        let pwd = env::current_dir()?;
        let mut command = Command::new("docker");

        command.args(&["run", "-u", &format!("{0}:{0}", geteuid())]);

        if self.variants.contains(TagVariants::GPU) {
            command.arg("--runtime=nvidia");
        }

        command.args(&["-it", "--rm", "-v", &format!("{}:/project", pwd.display())]).args(&[
            "-w",
            "/project",
            &String::from(self),
            cmd,
        ]);

        if let Some(args) = args {
            command.args(args);
        }

        eprintln!("{:?}", command);

        command.status().map(|_| ())
    }
}

impl<'a> From<&Image<'a>> for String {
    fn from(image: &Image) -> String {
        let mut buffer = ["tensorflow/tensorflow:", image.tag].concat();

        if !image.variants.is_empty() {
            if image.variants.contains(TagVariants::GPU) {
                buffer.push_str("-gpu");
            }

            if image.variants.contains(TagVariants::PY3) {
                buffer.push_str("-py3");
            }

            if image.variants.contains(TagVariants::JUPYTER) {
                buffer.push_str("-jupyter");
            }
        }

        buffer
    }
}

impl<'a> Display for Image<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("tensorflow/tensorflow:")?;
        f.write_str(self.tag)?;

        if !self.variants.is_empty() {
            f.write_char(':')?;

            let mut tag_found = false;
            let mut write_tag = move |tag: &'static str| -> fmt::Result {
                if tag_found {
                    f.write_char('-')?;
                }

                tag_found = true;
                f.write_str(tag)
            };

            if self.variants.contains(TagVariants::GPU) {
                write_tag("gpu")?;
            }

            if self.variants.contains(TagVariants::PY3) {
                write_tag("py3")?;
            }

            if self.variants.contains(TagVariants::JUPYTER) {
                write_tag("jupyter")?;
            }
        }

        Ok(())
    }
}