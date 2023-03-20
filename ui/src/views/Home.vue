<template>
  <div class="home">
    <el-card class="home-card">
      <div>PLAY AI</div>
      <el-button type="primary" @click="Gomoku">Gomoku</el-button>
    </el-card>
  </div>
</template>

<script>
import router from '@/router';

export default {
  name: 'HomeView',
  components: {
  },
  methods: {
    Gomoku() {
      router.push({ name: 'gomokuAI' })
    }
  }
}
</script>

<style scoped lang="stylus">
.home-card {
  margin: auto;
  max-width: 500px;
  min-width: 300px;
  padding: 10px;
}
</style>