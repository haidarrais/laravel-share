<?php

declare(strict_types=1);

namespace ArtisanShare\Binary;

/**
 * Resolves the platform-specific release asset name for the tunnel client.
 */
final class Platforms
{
    /**
     * Return the release asset filename for the current platform, or throw if
     * the platform is unsupported.
     *
     * @throws \RuntimeException
     */
    public static function assetName(): string
    {
        return self::assetNameFor(self::os(), self::arch());
    }

    /**
     * Return the release asset filename for a given OS and architecture pair.
     *
     * This is the contract with the release workflow, which uploads assets under
     * these names so the binary manager can fetch the right one per platform.
     *
     * @throws \RuntimeException for unsupported combinations
     */
    public static function assetNameFor(string $os, string $arch): string
    {
        return match ($os.'-'.$arch) {
            'linux-x64' => 'tunnel-client-linux-x86_64',
            'linux-arm64' => 'tunnel-client-linux-aarch64',
            'darwin-x64' => 'tunnel-client-darwin-x86_64',
            'darwin-arm64' => 'tunnel-client-darwin-aarch64',
            'windows-x64' => 'tunnel-client-windows-x86_64.exe',
            default => throw new \RuntimeException(
                "Artisan Share does not yet ship a binary for {$os}-{$arch}."
            ),
        };
    }

    /**
     * The current operating system family.
     */
    private static function os(): string
    {
        return match (PHP_OS_FAMILY) {
            'Linux' => 'linux',
            'Darwin' => 'darwin',
            'Windows' => 'windows',
            default => strtolower(PHP_OS_FAMILY),
        };
    }

    /**
     * The current CPU architecture.
     */
    private static function arch(): string
    {
        $arch = php_uname('m');

        return match (strtolower($arch)) {
            'x86_64', 'amd64' => 'x64',
            'aarch64', 'arm64' => 'arm64',
            default => strtolower($arch),
        };
    }
}
