<script lang="ts">
    export let title: string = "";
</script>

<div
    class="bg-white dark:bg-zinc-800 shadow rounded-lg p-6 mb-4 border border-zinc-200 dark:border-zinc-700"
>
    {#if title}
        <h3
            class="text-lg font-medium leading-6 text-zinc-900 dark:text-zinc-100 mb-4"
        >
            {title}
        </h3>
    {/if}
    <slot />
</div>
