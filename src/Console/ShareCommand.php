<?php

declare(strict_types=1);

namespace ArtisanShare\Console;

use ArtisanShare\Config\ConfigBuilder;
use ArtisanShare\Process\TunnelProcess;
use Illuminate\Console\Command;

class ShareCommand extends Command
{
    protected $signature = 'share
        {--driver= : Tunnel driver to use: relay, cloudflare, ssh}
        {--port= : Local port to forward inbound requests to}
        {--subdomain= : Requested subdomain (relay driver)}
        {--basic-auth= : "user:pass" HTTP basic auth for the public endpoint}
        {--inspector-port= : Port for the localhost web inspector (0 disables)}
        {--verbose : Show full request headers in the log}
        {--binary= : Path to an already-installed tunnel client binary}';

    protected $description = 'Share a local endpoint with a public URL via a tunnel driver you own';

    /**
     * Execute the share command.
     */
    public function handle(): int
    {
        $config = config('share');

        try {
            $builder = $this->builder($config);
            $builder->validate();
        } catch (\InvalidArgumentException $e) {
            $this->error($e->getMessage());

            return self::FAILURE;
        }

        // Resolve the binary: an explicit override, otherwise ensure one is cached.
        $binary = (string) ($this->option('binary') ?? '');
        if ($binary === '') {
            try {
                $binary = $builder->ensureBinary();
            } catch (\RuntimeException $e) {
                $this->error($e->getMessage());

                return self::FAILURE;
            }
        }

        $configPath = $builder->write();

        $this->line(sprintf(
            '<info>Artisan Share</info> driver=<comment>%s</comment> localhost:%s',
            $builder->driver(),
            (int) ($this->option('port') ?? $config['local_port'] ?? 8000)
        ));
        $this->line('Press <comment>Ctrl-C</comment> to stop.');

        $process = new TunnelProcess($binary, $configPath);

        try {
            $exit = $process->run();
        } catch (\RuntimeException $e) {
            $this->error($e->getMessage());

            return self::FAILURE;
        }

        return $exit === 0 ? self::SUCCESS : self::FAILURE;
    }

    /**
     * Build a config from the published configuration and command options.
     *
     * @param  array<string, mixed>  $config
     */
    private function builder(array $config): ConfigBuilder
    {
        return new ConfigBuilder($config, [
            'driver' => $this->option('driver'),
            'port' => $this->option('port'),
            'subdomain' => $this->option('subdomain'),
            'basic_auth' => $this->option('basic-auth'),
            'inspector-port' => $this->option('inspector-port'),
            'verbose' => (bool) $this->option('verbose'),
        ]);
    }
}
