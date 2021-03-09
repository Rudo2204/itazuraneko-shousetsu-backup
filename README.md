CI: [![Build Status](https://github.com/Rudo2204/itazuraneko-shousetsu-backup/workflows/CI/badge.svg)](https://github.com/Rudo2204/itazuraneko-shousetsu-backup/actions)\
License: [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

## Quickstart (it just works!:tm:)
1. Install [monolith](https://github.com/Y2Z/monolith) and add it to your PATH.
2. Run `itazuraneko_backup download` to start the downloading process.
3. Wait till the downloading process finishes. And done!

## Export
You can use the export subcommand to export data for various purposes.

There are two flags you can use:

The first flag is `--csv` which will export the data out to a friendly csv file. You then can use this exported csv file with `itazuraneko_backup` using the flag `--input` or whatever else you want.

The other one is `--job` which can be used to export all the jobs out if you want to run it on another machine with more cores/better internet connection using [GNU parallel](https://www.gnu.org/software/parallel/). You can invoke `cat jobs.txt | parallel --eta -j <num-jobs> {}` on the remote machine to achieve this.

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
