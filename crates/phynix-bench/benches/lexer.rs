use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use phynix_core::{LanguageKind, Strictness};
use phynix_lex::lex_into;
use std::hint::black_box;

fn bench_lex(c: &mut Criterion) {
    let bytes = 20 * 1024 * 1024;

    let big_comment = gen_big_comment(bytes);
    let big_heredoc = gen_big_heredoc(bytes);
    let big_html = gen_big_html(bytes);
    let big_realistic = gen_big_realistic(bytes);
    let big_string = gen_big_string(bytes);

    let inputs: &[(&str, &str)] = &[
        ("tiny", include_str!("../fixtures/lexer/tiny.php")),
        ("medium", include_str!("../fixtures/lexer/medium.php")),
        ("big_comment", big_comment.as_str()),
        ("big_heredoc", big_heredoc.as_str()),
        ("big_html", big_html.as_str()),
        ("big_realistic", big_realistic.as_str()),
        ("big_string", big_string.as_str()),
    ];

    let mut group = c.benchmark_group("lex_into");
    for (name, src) in inputs {
        group.throughput(Throughput::Bytes(src.len() as u64));

        group.bench_function(*name, |b| {
            let mut out = Vec::with_capacity(src.len());
            b.iter(|| {
                out.clear();
                lex_into(
                    black_box(src),
                    black_box(&LanguageKind::PhpCompat),
                    black_box(&Strictness::Lenient),
                    black_box(&mut out),
                )
                .unwrap();
                black_box(out.len());
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_lex);
criterion_main!(benches);

fn gen_big_comment(bytes: usize) -> String {
    let mut s = String::with_capacity(bytes + 16);
    s.push_str("<?php /*\n");
    while s.len() < bytes {
        s.push_str("*a");
    }
    s.push_str("\n*/\n");
    s
}

fn gen_big_heredoc(bytes: usize) -> String {
    const LABEL: &str = "LABEL1234567890";
    let mut s = String::with_capacity(bytes + 128);

    s.push_str("<?php\n$h = <<<");
    s.push_str(LABEL);
    s.push('\n');

    while s.len() < bytes {
        s.push_str("line\n  LABEL123456789X\n");
    }

    s.push_str(LABEL);
    s.push_str("\n;\n?>\n");
    s
}

fn gen_big_html(bytes: usize) -> String {
    let mut s = String::with_capacity(bytes + 64);

    while s.len() < bytes {
        s.push_str(
            "<div>Lorem ipsum dolor sit amet. 1234567890 ABC xyz</div>\n",
        );
    }

    s
}

fn gen_big_realistic(bytes: usize) -> String {
    let mut s = String::with_capacity(bytes + 256);

    const HTML: &str =
        "<div class='x'>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</div>\n";

    const PHP: &str = r#"<?php
/** Block 0 */
namespace N0\Sub0;
use A\B\C as C0;

final class K0 extends Base0 implements I0 {
  private int $v0 = 0;
  public function f0($x, $y) {
    $z = $x + $y * 0 - 0;
    if ($z >= 10 && $z <= 100) { $z++; } else { $z--; }
    foreach ([1,2,3,4,5] as $k => $v) { $z = ($z << 1) ^ $v; }
    $r = match ($z % 5) {
      0 => "a", 1 => "b", 2 => "c", 3 => "d", default => "e",
    };
    return $r . ":" . $z;
  }
}
?>
"#;

    while s.len() < bytes {
        for _ in 0..10 {
            s.push_str(HTML);
        }
        s.push_str(PHP);
    }

    s
}

fn gen_big_string(bytes: usize) -> String {
    let mut s = String::with_capacity(bytes + 64);

    s.push_str("<?php ");

    while s.len() < bytes {
        s.push_str("$s = \"");
        for _ in 0..4096 {
            s.push_str("\\\\");
        }
        s.push_str("\"; ");

        s.push_str("$t = '");
        for _ in 0..4096 {
            s.push('\\');
        }
        s.push_str("'; ?>\n<?php ");
    }

    s.push_str("?>\n");
    s
}
