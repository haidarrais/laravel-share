<?php

declare(strict_types=1);

namespace ArtisanShare\Tests\Binary;

use ArtisanShare\Binary\Platforms;
use ArtisanShare\Tests\TestCase;

class PlatformsTest extends TestCase
{
    public function test_it_maps_supported_platforms_to_release_assets(): void
    {
        $this->assertSame('tunnel-client-linux-x86_64', Platforms::assetNameFor('linux', 'x64'));
        $this->assertSame('tunnel-client-linux-aarch64', Platforms::assetNameFor('linux', 'arm64'));
        $this->assertSame('tunnel-client-darwin-x86_64', Platforms::assetNameFor('darwin', 'x64'));
        $this->assertSame('tunnel-client-darwin-aarch64', Platforms::assetNameFor('darwin', 'arm64'));
        $this->assertSame('tunnel-client-windows-x86_64.exe', Platforms::assetNameFor('windows', 'x64'));
    }

    public function test_it_rejects_an_unsupported_platform(): void
    {
        $this->expectException(\RuntimeException::class);

        Platforms::assetNameFor('linux', 'mips64');
    }
}
