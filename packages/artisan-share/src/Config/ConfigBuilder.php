<?php

declare(strict_types=1);

namespace ArtisanShare\Config;

use ArtisanShare\Binary\BinaryManager;

/**
 * Builds the JSON configuration file consumed by the Rust tunnel client.
 *
 * The JSON shape is the wire contract between this package and the binary; it
 * is intentionally plain so that any host language could emit it. This class
 * only maps the resolved command options and published `config/share.php`
 * settings onto that contract.
 */
final class ConfigBuilder
{
    public const DRIVERS = ['relay', 'cloudflare', 'ssh'];

    /**
     * @param  array<string, mixed>  $config  the resolved `share` config array
     * @param  array<string, mixed>  $options  overrides from the artisan command
     */
    public function __construct(
        private array $config,
        private array $options = [],
    ) {}

    /**
     * Validate the resolved driver name.
     *
     * @throws \InvalidArgumentException
     */
    public function validate(): void
    {
        $driver = $this->options['driver'] ?? $this->config['default'];

        if (! in_array($driver, self::DRIVERS, true)) {
            throw new \InvalidArgumentException(
                "Unsupported driver \"{$driver}\". Expected one of: ".implode(', ', self::DRIVERS).'.'
            );
        }
    }

    /**
     * Resolve the driver name after validation.
     */
    public function driver(): string
    {
        return (string) ($this->options['driver'] ?? $this->config['default']);
    }

    /**
     * Write the config JSON to a temporary file and return its path.
     *
     * @throws \RuntimeException
     */
    public function write(): string
    {
        $path = tempnam(sys_get_temp_dir(), 'artisan-share-');
        if ($path === false) {
            throw new \RuntimeException('Unable to create a temporary config file.');
        }

        $json = json_encode($this->toArray(), JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES);
        if ($json === false) {
            throw new \RuntimeException('Unable to encode tunnel client config as JSON.');
        }

        file_put_contents($path, $json);

        return $path;
    }

    /**
     * The resolved config as the JSON array expected by the tunnel client.
     *
     * @return array<string, mixed>
     */
    private function toArray(): array
    {
        $config = $this->config;
        $options = $this->options;
        $drivers = $config['drivers'] ?? [];
        $localPort = (int) ($options['port'] ?? $config['local_port'] ?? 8000);

        return [
            'driver' => $this->driver(),
            'local_port' => $localPort,
            'subdomain' => $options['subdomain'] ?? null,
            'basic_auth' => $options['basic_auth'] ?? null,
            'verbose' => (bool) ($options['verbose'] ?? false),
            'inspector_port' => (int) ($options['inspector-port'] ?? $config['inspector_port'] ?? 0),
            'relay' => [
                'endpoint' => $this->stringOption('relay.endpoint', $drivers, 'relay'),
                'token' => $this->stringOption('relay.token', $drivers, 'relay'),
            ],
            'cloudflare' => [
                'binary' => $this->stringOption('cloudflare.binary', $drivers, 'cloudflared'),
            ],
            'ssh' => [
                'host' => $this->stringOption('ssh.host', $drivers, ''),
                'user' => $this->stringOption('ssh.user', $drivers, ''),
                'remote_port' => (int) $this->mixedOption('ssh.remote_port', $drivers, 8080),
            ],
        ];
    }

    /**
     * Read a dotted key (e.g. `relay.endpoint`) from the drivers config.
     */
    private function stringOption(string $key, mixed $drivers, string $default): string
    {
        return (string) $this->mixedOption($key, $drivers, $default);
    }

    private function mixedOption(string $key, mixed $drivers, mixed $default): mixed
    {
        [$driver, $field] = explode('.', $key, 2);

        $value = $drivers[$driver][$field] ?? null;

        return ($value === null || $value === '') ? $default : $value;
    }

    /**
     * Ensure the tunnel client binary is available for the chosen driver.
     *
     * @throws \RuntimeException
     */
    public function ensureBinary(): string
    {
        return (new BinaryManager($this->config['binary'] ?? []))->ensure();
    }
}
