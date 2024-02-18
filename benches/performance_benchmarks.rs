use criterion::{black_box, criterion_group, criterion_main, Criterion};

// -- BYTE SIZE FORMATTING -- //

// note that sample file sizes ideally should match distribution of file sizes found on real
// machines. probs not evenly or normally distributed; maybe clustered around 1KB and 1 MB ig I had to guess
const EXAMPLE_BYTE_SIZES : [u64; 14] = [
    9,
    99,
    999,
    9999,
    99999,
    999999,
    9999999,
    99999999,
    999999999,
    9999999999,
    99999999999,
    999999999999,
    9999999999999,
    99999999999999,
];

fn format_bytes_with_log(num_bytes : u64) -> String {
    let num_digits = ((num_bytes as f64).log10() + 1.) as usize; // u64.ilog10 would make sense if my rust version was a bit newer
    let digits_after_decimal = (3 - (num_digits % 3)) % 3; // long winded negative modulus, to determine precision
    match num_digits {
        d if d < 4 => {
            format!("{} B", num_bytes)
        },
        d if d < 7 => {
            format!("{0:.1$} K", num_bytes as f64 / 1000., digits_after_decimal)
        },
        d if d < 10 => {
            format!("{0:.1$} M", num_bytes as f64 / 1000000., digits_after_decimal)
        },
        d if d < 13 => {
            format!("{0:.1$} G", num_bytes as f64 / 1000000000., digits_after_decimal)
        },
        _ => format!("over 1 T")
    }
}

fn format_bytes_with_cases(num_bytes : u64) -> String {
    const ONE_KB : u64 = 1000;
    const ONE_MB : u64 = 1000000;
    const ONE_GB : u64 = 1000000000;
    const ONE_TB : u64 = 1000000000000;

    match num_bytes {
        b if b < ONE_KB => {
            format!("{} B", num_bytes)
        },
        b if b < ONE_MB => {
            let num_digits_after_decimal = match num_bytes {
                b if b < ONE_KB * 10 => 2,
                b if b < ONE_KB * 100 => 1,
                _ => 0,
            };
            format!("{0:.1$} K", num_bytes as f64 / ONE_KB as f64, num_digits_after_decimal)
        },
        b if b < ONE_GB => {
            let num_digits_after_decimal = match num_bytes {
                b if b < ONE_MB * 10 => 2,
                b if b < ONE_MB * 100 => 1,
                _ => 0,
            };
            format!("{0:.1$} M", num_bytes as f64 / ONE_MB as f64, num_digits_after_decimal)
        },
        b if b < ONE_TB => {
            let num_digits_after_decimal = match num_bytes {
                b if b < ONE_GB * 10 => 2,
                b if b < ONE_GB * 100 => 1,
                _ => 0,
            };
            format!("{0:.1$} G", num_bytes as f64 / ONE_GB as f64, num_digits_after_decimal)
        },
        _ => format!("over 1 TB")
    }
}

fn format_bytes_with_loop(num_bytes: u64) -> String {
    const SUFFIXES : [&str; 4] = ["G", "M", "K", "B"];
    const MAGNITUDES : [u64; 4] = [
        u64::pow(10,9),
        u64::pow(10,6),
        u64::pow(10,3),
        1
    ];

    match num_bytes {
        b if b >= u64::pow(10,12) => {
            format!("over 1 TB")
        },
        _ => {
            let mut i = 0;
            while MAGNITUDES[i] > num_bytes {
                i += 1;
            }
            format!("{:.1} {}", num_bytes / MAGNITUDES[i], SUFFIXES[i])
        },
    }

}

fn compare_bytes_formatting(c: &mut Criterion) {
    c.bench_function(
        "format_bytes_with_loop",
        |b| b.iter(|| {
            for bytes in EXAMPLE_BYTE_SIZES.iter() {
                black_box( format_bytes_with_loop(*bytes) );
            }
        })
    );

    c.bench_function(
        "format_bytes_with_log",
        |b| b.iter(|| {
            for bytes in EXAMPLE_BYTE_SIZES.iter() {
                black_box( format_bytes_with_log(*bytes) );
            }
        })
    );

    c.bench_function(
        "format_bytes_with_cases",
        |b| b.iter(|| {
            for bytes in EXAMPLE_BYTE_SIZES.iter() {
                black_box( format_bytes_with_cases(*bytes) );
            }
        })
    );
}

// compare read_path_name vs read_path_name_size_and_date
// compare read_directory_contents_into_binary_heap vs read_directory_contents_into_vec_then_sort
// compare format_bytes vs format_date

criterion_group!(benches, compare_bytes_formatting);
criterion_main!(benches);
