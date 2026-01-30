<script lang="ts">
    import type { Job } from "$lib/types";
    import { JobState } from "$lib/types";
    import Table from "./ui/Table.svelte";
    import Badge from "./ui/Badge.svelte";

    export let jobs: Job[] = [];

    function getStateVariant(state: JobState) {
        switch (state) {
            case JobState.RUNNING:
                return "success";
            case JobState.PENDING:
                return "warning";
            case JobState.FAILED:
                return "danger";
            case JobState.CANCELLED:
                return "neutral";
            case JobState.COMPLETED:
                return "success"; // or neutral
            default:
                return "neutral";
        }
    }

    function formatDate(dateStr: string | null) {
        if (!dateStr) return "-";
        return new Date(dateStr).toLocaleString();
    }
</script>

<div
    class="bg-white dark:bg-zinc-800 shadow rounded-lg overflow-hidden border border-zinc-200 dark:border-zinc-700"
>
    <Table>
        <thead class="bg-zinc-50 dark:bg-zinc-900/50">
            <tr>
                <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                    >Job ID</th
                >
                <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                    >User</th
                >
                <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                    >Partition</th
                >
                <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                    >State</th
                >
                <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                    >Nodes</th
                >
                <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                    >Submit Time</th
                >
            </tr>
        </thead>
        <tbody
            class="bg-white dark:bg-zinc-800 divide-y divide-zinc-200 dark:divide-zinc-700"
        >
            {#each jobs as job}
                <tr>
                    <td
                        class="px-6 py-4 whitespace-nowrap text-sm font-medium text-zinc-900 dark:text-zinc-100"
                        >{job.job_id}</td
                    >
                    <td
                        class="px-6 py-4 whitespace-nowrap text-sm text-zinc-500 dark:text-zinc-400"
                        >{job.user}</td
                    >
                    <td
                        class="px-6 py-4 whitespace-nowrap text-sm text-zinc-500 dark:text-zinc-400"
                        >{job.partition}</td
                    >
                    <td class="px-6 py-4 whitespace-nowrap text-sm">
                        <Badge variant={getStateVariant(job.state)}
                            >{job.state}</Badge
                        >
                    </td>
                    <td
                        class="px-6 py-4 whitespace-nowrap text-sm text-zinc-500 dark:text-zinc-400"
                        >{job.num_nodes} ({job.num_cpus} CPUs)</td
                    >
                    <td
                        class="px-6 py-4 whitespace-nowrap text-sm text-zinc-500 dark:text-zinc-400"
                        >{formatDate(job.submit_time)}</td
                    >
                </tr>
            {/each}
            {#if jobs.length === 0}
                <tr>
                    <td
                        colspan="6"
                        class="px-6 py-4 text-center text-sm text-zinc-500 dark:text-zinc-400"
                        >No active jobs</td
                    >
                </tr>
            {/if}
        </tbody>
    </Table>
</div>
