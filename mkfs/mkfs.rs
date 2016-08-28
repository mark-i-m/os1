//! A simple tool for building OFS images

mod image;

use std::env::args;

use image::OFSImage;

pub fn main() {
    // read command line args
    let mut args = args();
    args.next(); // drop the first arg (./mkfs)

    // create a new image
    let mut image = OFSImage::new(image::SECTOR_SIZE * 8, image::SECTOR_SIZE * 8);

    // add the files to the image
    for arg in args {
        image.add_file(arg.as_str());
    }

    image.burn("hdd.img");
}
