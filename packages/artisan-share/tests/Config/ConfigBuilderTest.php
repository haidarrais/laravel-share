<?php

declare(strict_types=1);

namespace ArtisanShare\Tests\Config;

use ArtisanShare\Config\ConfigBuilder;
use ArtisanShare\Tests\TestCase;

class ConfigBuilderTest extends TestCase
{
    private array $baseConfig = [
        'default' => 'cloudflare',
        'drivers' => [
            'relay' => [
                'endpoint' => 'wss://tunnel.example.dev',
                'token' => 'sekret',
            ],
            'cloudflare' => [
                'binary' => 'cloudflared',
            ],
            'ssh' => [
                'host' => 'example.com',
                'user' => 'deploy',
                'remote_port' => 2222,
            ],
        ],
        'local_port' => 8000,
        'inspector_port' => 4040,
        'binary' => [
            'repo' => 'haidarrais/laravel-share',
        ],
    ];

    public function test_it_uses_the_default_driver_when_none_is_given(): void
    {
        $builder = new ConfigBuilder($this->baseConfig);

        $this->assertSame('cloudflare', $builder->driver());
    }

    public function test_it_resolves_an_explicit_driver(): void
    {
        $builder = new ConfigBuilder($this->baseConfig, ['driver' => 'relay']);

        $this->assertSame('relay', $builder->driver());
    }

    public function test_it_rejects_an_unknown_driver(): void
    {
        $this->expectException(\InvalidArgumentException::class);

        (new ConfigBuilder($this->baseConfig, ['driver' => 'nope']))->validate();
    }

    public function test_it_writes_a_valid_json_config_file(): void
    {
        $builder = new ConfigBuilder($this->baseConfig, ['driver' => 'relay']);
        $path = $builder->write();

        $decoded = json_decode((string) file_get_contents($path), true);
        @unlink($path);

        $this->assertIsArray($decoded);
        $this->assertSame('relay', $decoded['driver']);
        $this->assertSame(8000, $decoded['local_port']);
        $this->assertSame(4040, $decoded['inspector_port']);
        $this->assertFalse($decoded['verbose']);
        $this->assertSame('wss://tunnel.example.dev', $decoded['relay']['endpoint']);
        $this->assertSame('sekret', $decoded['relay']['token']);
    }

    public function test_it_applies_command_line_overrides(): void
    {
        $builder = new ConfigBuilder($this->baseConfig, [
            'driver' => 'relay',
            'port' => 9000,
            'subdomain' => 'my-app',
            'basic_auth' => 'user:pass',
            'inspector-port' => 0,
            'verbose' => true,
        ]);

        $path = $builder->write();
        $decoded = json_decode((string) file_get_contents($path), true);
        @unlink($path);

        $this->assertSame(9000, $decoded['local_port']);
        $this->assertSame('my-app', $decoded['subdomain']);
        $this->assertSame('user:pass', $decoded['basic_auth']);
        $this->assertSame(0, $decoded['inspector_port']);
        $this->assertTrue($decoded['verbose']);
    }
}
