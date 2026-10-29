<?php
// Template-managed (ext-template). Prints a canonical signature listing
// of either the loaded extension or the curated stubs, so CI can diff
// them: the stubs must declare exactly what the extension registers.
//
//   php -d extension=... stubs-check.php runtime whisper
//   php stubs-check.php stubs stubs/whisper.stubs.php 'Displace\Whisper\'
//
// Run the two modes in separate processes: the stubs declare the same
// class names the extension registers.

declare(strict_types=1);

[$_, $mode, $target] = $argv + [null, null, null];

if ($mode === 'runtime') {
    $classes = (new ReflectionExtension($target))->getClassNames();
} elseif ($mode === 'stubs') {
    $before = get_declared_classes();
    require $target;
    $prefix = $argv[3] ?? '';
    $classes = array_filter(
        array_diff(get_declared_classes(), $before),
        static fn (string $c): bool => str_starts_with($c, $prefix),
    );
} else {
    fwrite(STDERR, "usage: stubs-check.php runtime <ext> | stubs <file> <namespace-prefix>\n");
    exit(2);
}

$classes = array_values($classes);
sort($classes);
$out = [];
foreach ($classes as $name) {
    $c = new ReflectionClass($name);
    $interfaces = $c->getInterfaceNames();
    sort($interfaces);
    $out[] = sprintf(
        '%s %s extends %s implements %s',
        $c->isFinal() ? 'final class' : ($c->isAbstract() ? 'abstract class' : 'class'),
        $name,
        $c->getParentClass() ? $c->getParentClass()->getName() : '-',
        implode(',', $interfaces),
    );
    foreach ($c->getReflectionConstants() as $k) {
        if ($k->getDeclaringClass()->getName() === $name) {
            $out[] = '  const ' . $k->getName() . ' = ' . var_export($k->getValue(), true);
        }
    }
    $methods = $c->getMethods();
    usort($methods, static fn ($a, $b): int => strcmp($a->getName(), $b->getName()));
    foreach ($methods as $m) {
        if ($m->getDeclaringClass()->getName() !== $name) {
            continue;
        }
        $params = [];
        foreach ($m->getParameters() as $p) {
            $s = ($p->getType() ?? 'mixed') . ' ' . ($p->isPassedByReference() ? '&' : '')
                . ($p->isVariadic() ? '...' : '') . '$' . $p->getName();
            if ($p->isDefaultValueAvailable()) {
                $s .= ' = ' . var_export($p->getDefaultValue(), true);
            } elseif ($p->isOptional() && !$p->isVariadic()) {
                $s .= ' = <optional>';
            }
            $params[] = $s;
        }
        $out[] = sprintf(
            '  %s function %s(%s): %s',
            implode(' ', Reflection::getModifierNames($m->getModifiers())),
            $m->getName(),
            implode(', ', $params),
            $m->hasReturnType() ? (string) $m->getReturnType() : 'none',
        );
    }
}
echo implode("\n", $out), "\n";
