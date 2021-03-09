CI: [![Build Status](https://github.com/Rudo2204/rtend/workflows/CI/badge.svg)](https://github.com/Rudo2204/rtend/actions)\
License: [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

## Quickstart (it just works!:tm:)
1. Install [monolith](https://github.com/Y2Z/monolith) and add it to your PATH.
2. Run `itazuraneko_backup download`.
3. Wait. (The program will download a lot of files)
4. Done.

## Recommended workflow (Linux)
1. Install [monolith](https://github.com/Y2Z/monolith) and add it to your PATH.
2. Install [GNU parallel](https://www.gnu.org/software/parallel/) and add it to your PATH.
3. Run `itazuraneko_backup export --job`. This will make a `itazuraneko.csv` file that you can use for inspection or whatever else purposes, and also a `jobs.txt` file that contains all the jobs.
4. We then can run execute these jobs in parallel using gnu `parallel` by invoking `cat jobs.txt | parallel --eta -j 100 {}`. Now we wait.
5. Done!

## Usage
```
USAGE:
    itazuraneko_backup download [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input <CSV>    use prepared csv file

USAGE:
    itazuraneko_backup export [FLAGS] [OPTIONS]

FLAGS:
    -s, --csv        export itazuraneko data to csv file
    -h, --help       Prints help information
    -j, --job        export jobs file for running parallel
    -V, --version    Prints version information

OPTIONS:
    -i, --input <CSV>        use prepared csv file
    -o, --output <OUTPUT>    output file
```
