<!doctype html>
<html><body>
<div>Hello</div>
<?php
/** Docblock */
# hash
// line
/* block */
#[Attr]
namespace A\\B;
use Foo\\Bar as Baz;

function test($x) {
  $y = 0xFF + 0b1010 + 0o77 + 1_000;
  $f1 = 12.34;
  $f2 = .4e+2;
  $s1 = "dq \\ \" \\n";
  $s2 = 'sq \\ \'';
  $s3 = `bt \\ \``;
  $a = $x?->m() ?? $y;
  $b = $x === $y || $x !== $y && $x <=> $y;
  $c = $x <<= 1; $d = $x >>= 1; $e = $x **= 2; $g = $x ??= 3;
  $h = $x .= "x"; $i = $x += 1; $j = $x -= 1; $k = $x *= 2; $l = $x /= 2;
  $m = $x %= 2; $n = $x &= 1; $o = $x |= 1; $p = $x ^= 1;
  $arr = [1,2,3]; $obj = (object)[]; $t = $arr[0] ?? null;
  echo $s1, $s2, $s3;
}

$hd = <<<LABEL
line 1
line 2
LABEL;

$nd = <<<'NOD'
same
NOD;

?>
<?= "echo open" ?>
</body></html>
