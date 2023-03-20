<template>
  <div class="home">
    <el-card class="home-card">
      <div class="title">BOARD GAME</div>
      <br>
      <el-row>
        <el-col
          :span="10"
          :offset="0"
        >
          <el-card :body-style="{ padding: '0px' }" class="click_button" @click="Gomoku">
            <img
              src="gomoku.png"
              class="image"
            />
            <div class="game-type">
              <span>Gomoku AI</span>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </el-card>
  </div>
</template>

<script>
import router from '@/router';

export default {
  name: 'HomeView',
  components: {
  },
  setup(props) {
    function Gomoku() {
      router.push({ name: 'gomokuAI' });
    }

    return {
      Gomoku
    }
  }
}
</script>

<style scoped lang="stylus">
.title {
  display: block;
  letter-spacing: -2px;
  font-size: 2em;
  margin-top: 0.34em;
  margin-bottom: 0.67em;
  margin-left: 0;
  margin-right: 0;
  font-weight: bold;
  text-align: center;
}
.home-card {
  margin: auto;
  max-width: 500px;
  min-width: 300px;
  padding: 10px;
}
.game-type {
  display: block;
  font-size: 1.5em;
  margin-left: 0;
  margin-right: 0;
  font-weight: bold;
  padding: 14px;
  text-align: center;
}
.image {
  width: 100%;
  display: block;
  -webkit-user-drag: none;
}
.bottom {
  margin-top: 13px;
  line-height: 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.click_button {
  cursor: pointer;
}
.click_button:hover {
  box-shadow: 2px 2px 20px #ddd;
}
</style>