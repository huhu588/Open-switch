<script setup lang="ts">
/**
 * 通用 SVG 图标组件
 * 使用阿里 iconfont 的 Symbol 引用方式
 */
interface Props {
  name: string            // 图标名称（不含 icon- 前缀）
  size?: number | string  // 图标大小
  color?: string          // 图标颜色
  className?: string      // 额外的 CSS 类名
}

const props = withDefaults(defineProps<Props>(), {
  size: 16,
  color: 'currentColor',
  className: ''
})

// 计算图标大小样式
const iconSize = typeof props.size === 'number' ? `${props.size}px` : props.size
</script>

<template>
  <svg
    class="svg-icon"
    :class="className"
    :style="{
      width: iconSize,
      height: iconSize,
      fill: color,
      verticalAlign: 'middle'
    }"
    aria-hidden="true"
  >
    <use :xlink:href="`#icon-${name}`" />
  </svg>
</template>

<style scoped>
.svg-icon {
  display: inline-block;
  overflow: hidden;
  flex-shrink: 0;
}
</style>
