<script setup>
defineProps({
  moduleCards: { type: Array, required: true },
  completedCount: { type: Number, required: true },
  totalModules: { type: Number, required: true },
});
</script>

<template>
  <div class="vb-screen">
    <section class="vb-section-card">
      <div class="vb-section-head">
        <div>
          <p class="vb-kicker">培训与考核</p>
          <h2>完成课程，通过考核</h2>
        </div>
        <span class="vb-section-meta">{{ completedCount }}/{{ totalModules }} 完成</span>
      </div>

      <div class="vb-module-list">
        <article
          v-for="module in moduleCards"
          :key="module.id"
          class="vb-module-card"
          :data-state="module.state"
        >
          <div class="vb-module-head">
            <div>
              <strong>{{ module.title }}</strong>
              <p>{{ module.hint }}</p>
            </div>
            <span class="vb-badge" :data-status="module.status">{{ module.status }}</span>
          </div>
          <div class="vb-module-foot">
            <small>{{ module.duration }}</small>
            <button class="vb-text-link" type="button">进入课程</button>
          </div>
        </article>
      </div>
    </section>
  </div>
</template>
