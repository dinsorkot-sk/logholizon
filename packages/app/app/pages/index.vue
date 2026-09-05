<template>
  <UContainer class="py-16">
    <UCard>
      <template #header>
        <h1 class="text-xl font-semibold">
          ERP setup ready
        </h1>
      </template>
      <p class="text-sm text-gray-500">
        Nuxt 4 + Nuxt UI gateway is running. Rust core owns persistence.
      </p>
      <div class="mt-4">
        <UButton to="/admin/meta/entity">
          Entity Manager
        </UButton>
      </div>
    </UCard>
  </UContainer>
</template>
