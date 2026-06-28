<script setup lang="ts">
import { useSettings } from '../composables/useSettings'

const {
  settings,
  loaded,
  theme,
  fontFamily,
  fontSize,
  lineHeight,
  margin,
} = useSettings()
</script>

<template>
  <div class="p-6 max-w-md mx-auto">
    <h1 class="text-xl font-bold mb-4 flex items-center">
      <span class="material-symbols-outlined mr-2">settings</span>
      Settings
    </h1>
    
    <div v-if="loaded" class="flex flex-col gap-4">
      <!-- Theme Setting -->
      <div class="flex flex-col gap-1">
        <label for="theme-select" class="font-medium flex items-center gap-1">
          <span class="material-symbols-outlined text-lg">palette</span>
          Theme
        </label>
        <select id="theme-select" v-model="theme" class="border p-1 rounded">
          <option value="light">Light</option>
          <option value="dark">Dark</option>
          <option value="sepia">Sepia</option>
        </select>
      </div>

      <!-- Font Family Setting -->
      <div class="flex flex-col gap-1">
        <label for="font-family-select" class="font-medium flex items-center gap-1">
          <span class="material-symbols-outlined text-lg">font_download</span>
          Font Family
        </label>
        <select id="font-family-select" v-model="fontFamily" class="border p-1 rounded">
          <option value="Literata">Literata</option>
          <option value="Georgia">Georgia</option>
          <option value="Inter">Inter</option>
          <option value="Outfit">Outfit</option>
        </select>
      </div>

      <!-- Font Size Setting -->
      <div class="flex flex-col gap-1">
        <label for="font-size-input" class="font-medium flex items-center gap-1">
          <span class="material-symbols-outlined text-lg">format_size</span>
          Font Size ({{ fontSize }}px)
        </label>
        <input
          id="font-size-input"
          type="range"
          v-model.number="fontSize"
          min="12"
          max="48"
          class="w-full"
        />
      </div>

      <!-- Line Height Setting -->
      <div class="flex flex-col gap-1">
        <label for="line-height-input" class="font-medium flex items-center gap-1">
          <span class="material-symbols-outlined text-lg">format_line_spacing</span>
          Line Spacing ({{ lineHeight }}x)
        </label>
        <input
          id="line-height-input"
          type="range"
          v-model.number="lineHeight"
          min="1.0"
          max="3.0"
          step="0.1"
          class="w-full"
        />
      </div>

      <!-- Margin Setting -->
      <div class="flex flex-col gap-1">
        <label for="margin-input" class="font-medium flex items-center gap-1">
          <span class="material-symbols-outlined text-lg">margin</span>
          Page Margin ({{ margin }}x)
        </label>
        <input
          id="margin-input"
          type="range"
          v-model.number="margin"
          min="0.5"
          max="3.0"
          step="0.1"
          class="w-full"
        />
      </div>

      <!-- Preview block -->
      <div class="mt-4 p-4 border rounded" :style="{ fontSize: fontSize + 'px' }">
        <p
          :style="{
            fontFamily: settings.fontFamily,
            fontSize: settings.fontSize + 'px',
            lineHeight: settings.lineHeight,
            paddingLeft: (settings.margin * 10) + 'px',
            paddingRight: (settings.margin * 10) + 'px'
          }"
        >
          Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aliquam ac ante eget sem congue dictum nec vel mi. Pellentesque egestas scelerisque nisl eget tincidunt. Morbi quis nisi nec metus commodo tempor in et lectus. Ut egestas, magna ac placerat feugiat, nulla elit fermentum turpis, ut euismod leo velit vel nisi. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Mauris congue arcu nisl, non tempor lorem ornare non. Curabitur elementum lacus felis, sit amet facilisis tellus egestas at. Sed dictum convallis sapien, vitae tincidunt neque tincidunt nec.

        </p>
      </div>
    </div>
    
    <div v-else class="text-gray-500">
      Loading settings...
    </div>
  </div>
</template>
