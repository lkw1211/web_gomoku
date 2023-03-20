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
  letter-spacing: -0.3vmin;
  font-size: 5vmin;
  margin-top: 0.2vmin;
  margin-bottom: 0.5vmin;
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
  font-size: 3.5vmin;
  margin-left: 0;
  margin-right: 0;
  font-weight: bold;
  padding: 2vmin;
  text-align: center;
}
.image {
  width: 100%;
  display: block;
  -webkit-user-drag: none;
}
.click_button {
  cursor: pointer;
}
.click_button:hover {
  box-shadow: 2px 2px 20px #ddd;
}
</style>