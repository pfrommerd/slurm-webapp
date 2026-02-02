<script setup lang="ts">
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Field, FieldDescription, FieldGroup, FieldLabel, FieldError } from '@/components/ui/field';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { IconDatabase } from '@tabler/icons-vue';
import { Separator } from '@/components/ui/separator';
import { useRoute } from 'vue-router';
import { computed } from 'vue';

const route = useRoute();
const isLogin = computed(() => route.path === '/login');
const isSignup = computed(() => route.path === '/signup');
</script>

<template>
  <header class="flex h-(--header-height) shrink-0 items-center gap-2 border-b transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height)">
    <div class="flex w-full items-center gap-1 px-4 lg:gap-2 lg:px-6">
      <router-link to="/" class="flex items-center gap-2">
        <IconDatabase class="!size-5" />
        <span class="text-base font-semibold">Cluster Monitor</span>
      </router-link>
      <Separator
        orientation="vertical"
        class="mx-2 data-[orientation=vertical]:h-4"
      />
      <h1 class="text-base font-medium min-w-24">
        {{ route.name }}
      </h1>
    </div>
  </header>
  <div class="flex min-h-svh w-full items-center justify-center p-6 md:p-10">
    <div class="w-full max-w-sm">
      <Card v-if="isLogin">
        <CardHeader>
            <CardTitle>Login to your account</CardTitle>
            <CardDescription>
            Enter your email below to login to your account
            </CardDescription>
        </CardHeader>
        <CardContent>
            <form>
            <FieldGroup v-if="isLogin">
                <Field>
                <FieldLabel for="email">
                    Email
                </FieldLabel>
                <Input
                    id="email"
                    type="email"
                    placeholder="m@example.com"
                    required
                />
                </Field>
                <Field>
                <div class="flex items-center">
                    <FieldLabel for="password">
                    Password
                    </FieldLabel>
                    <a
                    href="#"
                    class="ml-auto inline-block text-sm underline-offset-4 hover:underline"
                    >
                    Forgot your password?
                    </a>
                </div>
                <Input id="password" type="password" required />
                </Field>
                <Field>
                <Button type="submit">
                    Login
                </Button>
                <Button variant="outline" type="button">
                    Login with Google
                </Button>
                <FieldDescription class="text-center">
                    Don't have an account?
                    <Button variant="link"><router-link to="/signup">Create an account</router-link></Button>
                </FieldDescription>
                </Field>
            </FieldGroup>
            </form>
        </CardContent>
        </Card>
        <Card v-if="isSignup">
            <CardHeader>
                <CardTitle>Create a new account.</CardTitle>
                <CardDescription>
                Enter your email below to create a new account
                </CardDescription>
            </CardHeader>
            <CardContent>
            <form>
            <FieldGroup v-if="isSignup">
                <Field>
                <FieldLabel for="email">
                    Email
                </FieldLabel>
                <Input
                    id="email"
                    type="email"
                    placeholder="m@example.com"
                    required
                />
                </Field>
                <Field>
                <FieldLabel for="password">
                Password
                </FieldLabel>
                <Input id="password" type="password" required />
                <FieldLabel for="confirm_password">
                Confirm Password
                </FieldLabel>
                <Input id="confirm_password" type="password" />
                <FieldError>Passwords do not match.</FieldError>
                </Field>
                <Field>
                <Button type="submit">
                    Create Account
                </Button>
                <Button variant="outline" type="button">
                    Sign up with Google
                </Button>
                <FieldDescription class="text-center">
                    Already have an account?
                    <Button variant="link"><router-link to="/login">Login instead</router-link></Button>
                </FieldDescription>
                </Field>
            </FieldGroup>

            </form>
        </CardContent>
        </Card>
    </div>
  </div>
</template>