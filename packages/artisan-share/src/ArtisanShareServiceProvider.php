<?php

declare(strict_types=1);

namespace ArtisanShare;

use ArtisanShare\Console\ShareCommand;
use Illuminate\Support\ServiceProvider;

class ArtisanShareServiceProvider extends ServiceProvider
{
    /**
     * Register the package's services and configuration.
     */
    public function register(): void
    {
        $this->mergeConfigFrom(__DIR__.'/../config/share.php', 'share');
    }

    /**
     * Bootstrap the package's services, publishable assets, and commands.
     */
    public function boot(): void
    {
        if ($this->app->runningInConsole()) {
            $this->publishes([
                __DIR__.'/../config/share.php' => $this->app->configPath('share.php'),
            ], 'share-config');

            $this->commands([
                ShareCommand::class,
            ]);
        }
    }
}
