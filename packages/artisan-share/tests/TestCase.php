<?php

declare(strict_types=1);

namespace ArtisanShare\Tests;

use ArtisanShare\ArtisanShareServiceProvider;
use Illuminate\Foundation\Application;
use Orchestra\Testbench\TestCase as BaseTestCase;

abstract class TestCase extends BaseTestCase
{
    /**
     * Load the package service provider.
     *
     * @param  Application  $app
     * @return array<int, class-string>
     */
    protected function getPackageProviders($app): array
    {
        return [
            ArtisanShareServiceProvider::class,
        ];
    }
}
