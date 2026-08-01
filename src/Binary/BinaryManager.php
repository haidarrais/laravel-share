<?php

declare(strict_types=1);

namespace ArtisanShare\Binary;

use GuzzleHttp\Client as HttpClient;

/**
 * Locates, downloads, verifies, and caches the Rust tunnel client binary.
 *
 * On first run the binary is fetched from the GitHub release asset matching the
 * current platform, its SHA-256 checksum is verified, and it is cached under
 * `~/.artisan-share/bin`. Subsequent runs reuse the cached copy.
 */
final class BinaryManager
{
    private HttpClient $http;

    /**
     * @param  array<string, mixed>  $config  the `share.binary` config block
     */
    public function __construct(private array $config = [])
    {
        $this->http = new HttpClient([
            'timeout' => 60,
            'connect_timeout' => 10,
        ]);
    }

    /**
     * Return the path to a verified, executable tunnel client binary, fetching
     * it if necessary.
     *
     * @throws \RuntimeException when the binary cannot be obtained
     */
    public function ensure(): string
    {
        $cacheDir = $this->cacheDir();
        $path = $cacheDir.DIRECTORY_SEPARATOR.self::binaryFilename();

        if (is_file($path) && $this->isExecutable($path)) {
            return $path;
        }

        if (! is_dir($cacheDir) && ! @mkdir($cacheDir, 0o755, true) && ! is_dir($cacheDir)) {
            throw new \RuntimeException("Unable to create binary cache directory: {$cacheDir}");
        }

        $this->download($path);

        if (! is_file($path) || ! $this->isExecutable($path)) {
            throw new \RuntimeException("Downloaded binary is missing or not executable: {$path}");
        }

        return $path;
    }

    /**
     * The absolute path of the binary cache directory.
     */
    private function cacheDir(): string
    {
        if (! empty($this->config['cache_dir'])) {
            return rtrim((string) $this->config['cache_dir'], '/\\');
        }

        return rtrim(getenv('HOME') ?: sys_get_temp_dir(), '/\\').'/.artisan-share/bin';
    }

    /**
     * Download and verify the binary for the current platform.
     *
     * @throws \RuntimeException
     */
    private function download(string $path): void
    {
        $url = $this->downloadUrl();
        $checksum = $this->fetchChecksum($url);

        $temp = $path.'.download';

        try {
            $this->http->get($url, ['sink' => $temp]);

            if (! is_file($temp)) {
                throw new \RuntimeException("Download produced no file: {$url}");
            }

            $actual = hash_file('sha256', $temp);
            if ($actual !== $checksum) {
                @unlink($temp);
                throw new \RuntimeException(
                    "Checksum mismatch for {$url}: expected {$checksum}, got {$actual}."
                );
            }

            if (! @rename($temp, $path)) {
                throw new \RuntimeException("Unable to move downloaded binary into place: {$path}");
            }

            @chmod($path, 0o755);
        } finally {
            @unlink($temp);
        }
    }

    /**
     * Build the download URL for the current platform's asset.
     */
    private function downloadUrl(): string
    {
        $repo = $this->config['repo'] ?? 'haidarrais/laravel-share';
        $tag = $this->config['tag'] ?? 'v0.1.3';

        return sprintf(
            'https://github.com/%s/releases/download/%s/%s',
            $repo,
            $tag,
            Platforms::assetName()
        );
    }

    /**
     * Fetch the expected SHA-256 checksum for an asset. We publish a per-asset
     * `*.sha256` sidecar alongside releases.
     *
     * @throws \RuntimeException
     */
    private function fetchChecksum(string $assetUrl): string
    {
        $checksumUrl = $assetUrl.'.sha256';

        try {
            $response = $this->http->get($checksumUrl);
        } catch (\Throwable $e) {
            throw new \RuntimeException(
                "Unable to fetch checksum {$checksumUrl}: {$e->getMessage()}",
                0,
                $e
            );
        }

        $contents = trim((string) $response->getBody());

        return strtok($contents, " \t\n") ?: '';
    }

    /**
     * Whether a cached binary is ready to execute.
     */
    private function isExecutable(string $path): bool
    {
        return is_file($path) && is_executable($path);
    }

    /**
     * The on-disk filename for the cached binary.
     */
    private static function binaryFilename(): string
    {
        return str_ends_with(Platforms::assetName(), '.exe')
            ? 'tunnel-client.exe'
            : 'tunnel-client';
    }
}
