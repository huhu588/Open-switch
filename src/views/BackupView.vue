<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import SvgIcon from '@/components/SvgIcon.vue'

const { t } = useI18n()
</script>

<template>
  <div class="max-w-2xl mx-auto">
    <div class="rounded-xl bg-surface/30 border border-border p-6">
      <div class="flex items-center gap-3 mb-6">
        <SvgIcon name="save" :size="32" class="text-accent" />
        <h2 class="text-xl font-semibold">{{ t('backup.title') }}</h2>
      </div>

      <div class="space-y-6">
        <section>
          <h3 class="font-medium mb-2">{{ t('backup.featureTitle') }}</h3>
          <p class="text-sm text-muted-foreground">
            {{ t('backup.featureDesc') }}
          </p>
        </section>

        <section>
          <h3 class="font-medium mb-2">{{ t('backup.supportedTypes') }}</h3>
          <ul class="text-sm text-muted-foreground space-y-1">
            <li>• {{ t('backup.openCodeConfig') }}</li>
            <li>• {{ t('backup.mcpConfig') }}</li>
          </ul>
        </section>

        <section>
          <h3 class="font-medium mb-2">{{ t('backup.usage') }}</h3>
          <p class="text-sm text-muted-foreground mb-3">
            {{ t('backup.usageDesc') }}
          </p>
          <div class="bg-surface rounded-lg p-4 font-mono text-sm space-y-2">
            <div class="flex items-center gap-2">
              <span class="text-accent-500">$</span>
              <span>openswitch backup</span>
              <span class="text-muted-foreground ml-4"># {{ t('backup.createBackup') }}</span>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-accent-500">$</span>
              <span>openswitch restore</span>
              <span class="text-muted-foreground ml-4"># {{ t('backup.restoreBackup') }}</span>
            </div>
          </div>
        </section>

        <section class="pt-4 border-t border-border">
          <div class="flex items-start gap-3 text-sm">
            <SvgIcon name="info" :size="20" class="text-accent flex-shrink-0 mt-0.5" />
            <div>
              <p class="font-medium">{{ t('backup.tipTitle') }}</p>
              <p class="text-muted-foreground">
                {{ t('backup.tipContent') }}
              </p>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>
