<template>
    <div id="header-comp">
        <el-row class="header-comp">
            <el-col
                :span="6"
                style="text-align: left;">
                <img
                    src="home.png"
                    class="home"
                    @click="home">
            </el-col>
            <el-col
                :span="12">
                <div class="title">{{ title }}</div>
            </el-col>
            <el-col
                :span="6"
                style="text-align: right;">
                <img
                    src="setting.png"
                    class="setting"
                    @click="setting">
            </el-col>
        </el-row>
        <el-dialog
            v-model="state.settingVisible"
            title="Setting"
            :width="state.dialogWidth"
            :close-on-click-modal="false"
            :close-on-press-escape="false"
        >
            <div>
                <el-row>
                    <el-col
                        :span="6">
                        <span class="demonstration">TimeLimit</span>
                    </el-col>
                    <el-col
                        :span="14">
                        <div>
                            <el-slider class="slider" v-model="state.timelimit" :step="5" :min="5" :max="60" show-stops @change="$emit('timelimitChange', state.timelimit)"/>
                        </div>
                    </el-col>
                    <el-col
                        :span="4"
                        class="demonstration left-spacing">
                        {{ state.timelimit }} S
                    </el-col>
                </el-row>
            </div>
        </el-dialog>
    </div>
</template>
  
<script>
import router from '@/router';
import { reactive } from 'vue';

export default {
    name: 'HeaderComp',
    props: ['title'],
    setup(props) {
        const state = reactive({
            settingVisible: false,
            timelimit: 20,
            dialogWidth: window.innerWidth < 500 ? '100%' : '500px',
        });

        window.onresize = () => {
            state.dialogWidth = window.innerWidth < 500 ? '100%' : '500px';
        }

        function home() {
            router.push({ name: 'home' });
        };

        function setting() {
            state.settingVisible = true;
        }

        return {
            home,
            setting,
            state
        };
    }
}
</script>

<style scoped lang="stylus">


@media	(min-aspect-ratio: 8/10) {
    .home {
        margin-top: 1.5vmin;
        width: 6vmin;
        height: 6vmin;
        margin-left: 3vmin;
        cursor: pointer;
    }
    .title {
        display: block;
        font-size: 4vmin;
        margin-top: 1vmin;
        margin-bottom: 1vmin;
        margin-left: 0;
        margin-right: 0;
        font-weight: bold;
    }
    .header-comp {
        width: calc(82vmin - 50px);
        position: relative;
        display: inline-flex;
    }

    .setting {
        margin-top: 1.5vmin;
        width: 6vmin;
        height: 6vmin;
        margin-right: 3vmin;
        cursor: pointer;
    }
    .demonstration {
        font-size: 2vmin;
        line-height: 4vmin;
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        margin-bottom: 0;
    }

    .left-spacing {
        letter-spacing: -0.2vmin;
    }
    .slider {
        flex: 0 0 70%;
    }
}
@media	(max-aspect-ratio: 799/1000) {
    .home {
        margin-top: 1.5vmin;
        width: 7vmin;
        height: 7vmin;
        margin-left: 3vmin;
        cursor: pointer;
    }
    .title {
        display: block;
        font-size: 5vmin;
        margin-top: 1vmin;
        margin-bottom: 1vmin;
        margin-left: 0;
        margin-right: 0;
        font-weight: bold;
    }
    .header-comp {
        width: calc(100vw - 5px);
        position: relative;
        display: inline-flex;
    }

    .setting {
        margin-top: 1.5vmin;
        width: 7vmin;
        height: 7vmin;
        margin-right: 3vmin;
        cursor: pointer;
    }
    .demonstration {
        font-size: 2vmin;
        line-height: 4vmin;
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        margin-bottom: 0;
    }

    .left-spacing {
        letter-spacing: -0.2vmin;
    }
    .slider {
        flex: 0 0 70%;
    }
}
</style>