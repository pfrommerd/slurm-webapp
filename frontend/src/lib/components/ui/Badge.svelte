<script lang="ts">
    export let variant:
        | "default"
        | "success"
        | "warning"
        | "danger"
        | "neutral" = "default";
    export let size: "sm" | "md" | "lg" = "md";

    const styles = {
        default:
            "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300",
        success:
            "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300",
        warning:
            "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300",
        danger: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300",
        neutral:
            "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300",
    };
    const sizes = {
        sm: "px-2 py-1 text-xs",
        md: "px-2.5 py-0.5 text-sm",
        lg: "px-3 py-1 text-base",
    };
</script>

<span
    class={`inline-flex items-center ${sizes[size]} font-medium ${styles[variant]}`}
>
    <slot />
</span>
