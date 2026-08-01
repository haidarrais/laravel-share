<?php

declare(strict_types=1);

namespace ArtisanShare\Process;

/**
 * Manages the lifecycle of the tunnel client subprocess.
 *
 * The process is spawned with the tunnel client binary and a path to the JSON
 * config file, with its output streamed to the console. On shutdown (Ctrl-C,
 * command abort, or natural exit) the process is terminated and its temporary
 * config file is removed.
 */
final class TunnelProcess
{
    /** @var resource|null */
    private $process = null;

    private array $pipes = [];

    private int $pendingSignal = 0;

    public function __construct(
        private string $binary,
        private string $configPath,
    ) {
        if (function_exists('pcntl_async_signals')) {
            pcntl_async_signals(true);
            pcntl_signal(SIGINT, $this->onSignal(...));
            pcntl_signal(SIGTERM, $this->onSignal(...));
        }
    }

    /**
     * Start the tunnel client and stream its output until it exits.
     *
     * @throws \RuntimeException when the process cannot be started
     */
    public function run(): int
    {
        $command = [$this->binary, '--config', $this->configPath];

        $descriptors = [
            0 => ['pipe', 'r'], // stdin: unused, but present so the child inherits a handle
            1 => ['pipe', 'w'], // stdout
            2 => ['pipe', 'w'], // stderr
        ];

        $this->process = proc_open($command, $descriptors, $this->pipes);

        if (! is_resource($this->process)) {
            throw new \RuntimeException(
                'Unable to launch tunnel client: '.implode(' ', $command)
            );
        }

        fclose($this->pipes[0]); // we never write to stdin

        stream_set_blocking($this->pipes[1], false);
        stream_set_blocking($this->pipes[2], false);

        $status = 0;

        try {
            $status = $this->pump();
        } finally {
            $this->cleanup();
        }

        return $status;
    }

    /**
     * Read and forward both output streams until the process exits, terminating
     * it if the parent is interrupted.
     */
    private function pump(): int
    {
        while (true) {
            if ($this->hasBeenInterrupted()) {
                $this->terminate();

                return 130;
            }

            $this->drain($this->pipes[1], STDOUT);
            $this->drain($this->pipes[2], STDERR);

            $status = proc_get_status($this->process);
            if ($status === false) {
                break;
            }

            if (! $status['running']) {
                // Flush any remaining buffered output.
                $this->drain($this->pipes[1], STDOUT);
                $this->drain($this->pipes[2], STDERR);

                return $status['exitcode'] ?? 0;
            }

            usleep(50_000);
        }

        return 0;
    }

    /**
     * Copy any pending output from a pipe to a stream.
     *
     * @param  resource  $pipe
     * @param  resource  $out
     */
    private function drain($pipe, $out): void
    {
        if (! is_resource($pipe)) {
            return;
        }

        while (($line = fgets($pipe)) !== false) {
            fwrite($out, $line);
        }
    }

    /**
     * Whether a SIGINT/SIGTERM has been delivered to the parent process.
     */
    private function hasBeenInterrupted(): bool
    {
        return $this->pendingSignal > 0;
    }

    /**
     * Record a received signal for graceful shutdown.
     */
    private function onSignal(int $signo): void
    {
        $this->pendingSignal = $signo;
    }

    /**
     * Forcefully terminate the child process.
     */
    private function terminate(): void
    {
        if (! is_resource($this->process)) {
            return;
        }

        if (function_exists('proc_terminate')) {
            proc_terminate($this->process);
            usleep(200_000);
        }
    }

    /**
     * Close pipes and the process handle.
     */
    private function cleanup(): void
    {
        foreach ($this->pipes as $pipe) {
            if (is_resource($pipe)) {
                fclose($pipe);
            }
        }

        if (is_resource($this->process)) {
            proc_close($this->process);
        }

        @unlink($this->configPath);
    }
}
