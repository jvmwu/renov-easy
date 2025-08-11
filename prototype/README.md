# 家庭装修接单 App - 高保真原型文档

## 项目概述

本项目是一个基于地理位置的双端服务平台原型，连接需要装修服务的用户和专业装修工人。采用 HTML5 + Tailwind CSS 技术栈开发，严格遵循 iOS Human Interface Guidelines 设计规范，针对 iPhone 16 Pro 进行了专门优化。

## 技术栈

- **前端框架**: HTML5 + CSS3
- **样式框架**: Tailwind CSS 4.1.11
- **图标库**: FontAwesome 6.4.0
- **地图服务**: Google Maps JavaScript API
- **设备适配**: iPhone 16 Pro (393×852 points)
- **设计规范**: iOS Human Interface Guidelines

## 项目结构

```
UI/
├── index.html                 # 主入口页面，展示所有原型
├── styles/
│   ├── design-system.css     # 设计系统和通用样式
│   └── config.json          # 样式配置数据
├── auth/                     # 认证系统目录
│   ├── auth-entry.html       # 登录注册入口
│   ├── phone-auth.html       # 手机号验证
│   ├── sms-verification.html # 短信验证
│   └── user-type-selection.html # 用户类型选择
├── user-home.html           # 用户端 - 装修首页
├── user-nearby.html         # 用户端 - 附近页面
├── user-profile.html        # 用户端 - 个人中心
├── worker-home.html         # 装修工端 - 地图首页
├── worker-orders.html       # 装修工端 - 订单管理
├── worker-profile.html      # 装修工端 - 个人中心
└── README.md               # 项目文档
```

## 页面功能说明

### 认证系统界面

#### 1. 认证入口页面 (auth/auth-entry.html)

- **核心功能**: 应用启动时的登录注册入口
- **主要组件**:
  - 应用 Logo 和品牌标语展示
  - 手机号登录/注册按钮
  - 社交登录选项（Apple、Google、Facebook）
  - 用户协议和隐私政策链接
- **交互流程**: 选择认证方式 → 跳转到相应认证流程

#### 2. 手机号验证页面 (auth/phone-auth.html)

- **核心功能**: 手机号输入和验证码发送
- **主要组件**:
  - 国家代码选择器（默认+86）
  - 手机号格式验证输入框
  - 获取验证码按钮
  - 社交登录快捷入口
- **交互流程**: 输入手机号 → 格式验证 → 发送验证码 → 跳转验证页面

#### 3. 短信验证页面 (auth/sms-verification.html)

- **核心功能**: 6 位短信验证码输入和验证
- **主要组件**:
  - 6 位数字验证码输入框
  - 自动焦点切换和粘贴支持
  - 60 秒倒计时重发功能
  - 验证成功/失败状态反馈
- **交互流程**: 输入验证码 → 实时验证 → 成功后跳转用户类型选择

#### 4. 用户类型选择页面 (auth/user-type-selection.html)

- **核心功能**: 新用户选择账户类型（普通用户/装修工）
- **主要组件**:
  - 两种用户类型的选择卡片
  - 功能特性展示
  - 装修工认证提示
  - 选择确认按钮
- **交互流程**: 选择用户类型 → 确认选择 → 跳转到相应主页面

### 用户端界面

#### 1. 装修首页 (user-home.html)

- **核心功能**: 地图选点发布装修需求
- **主要组件**:
  - Google Maps 地图容器
  - 位置搜索和自动补全
  - 底部抽屉式订单表单
  - 装修类型选择器
  - 预算范围滑块
  - 照片上传组件
- **交互流程**: 地图选点 → 填写需求 → 上传照片 → 提交订单

#### 2. 附近页面 (user-nearby.html)

- **核心功能**: 浏览附近装修工信息
- **主要组件**:
  - 装修工列表卡片
  - 筛选功能面板
  - 装修工详情模态框
  - 作品展示轮播
  - 收藏和联系功能
- **交互流程**: 浏览列表 → 查看详情 → 收藏/联系

#### 3. 个人中心 (user-profile.html)

- **核心功能**: 订单管理和账户设置
- **主要组件**:
  - 用户信息展示
  - 订单状态标签页
  - 订单详情查看
  - 收藏装修工管理
  - 账户设置表单
- **交互流程**: 查看订单 → 管理收藏 → 设置账户

### 装修工端界面

#### 1. 地图首页 (worker-home.html)

- **核心功能**: 地图查看和接单
- **主要组件**:
  - 订单标记地图
  - 筛选条件面板
  - 订单详情卡片
  - 接单操作按钮
  - 统计数据展示
- **交互流程**: 查看订单 → 筛选条件 → 接单操作

#### 2. 订单管理 (worker-orders.html)

- **核心功能**: 订单状态管理和进度更新
- **主要组件**:
  - 订单状态标签页
  - 订单列表卡片
  - 进度更新模态框
  - 客户沟通界面
  - 施工照片上传
- **交互流程**: 管理订单 → 更新进度 → 完成验收

#### 3. 个人中心 (worker-profile.html)

- **核心功能**: 资质展示和收入统计
- **主要组件**:
  - 个人资质卡片
  - 收入统计图表
  - 作品集管理
  - 客户评价展示
  - 账户设置
- **交互流程**: 展示资质 → 管理作品 → 查看收入

## 设计系统

### 色彩规范

```css
--primary-color: #007aff; /* iOS蓝 - 主色调 */
--success-color: #34c759; /* 绿色 - 成功状态 */
--warning-color: #ff3b30; /* 红色 - 警告状态 */
--neutral-color: #8e8e93; /* 灰色 - 次要信息 */
--background-color: #f2f2f7; /* 浅灰 - 背景色 */
--surface-color: #ffffff; /* 白色 - 卡片背景 */
```

### 字体规范

- **字体族**: SF Pro Display/Text, -apple-system
- **标题**: 24-32px, 粗体
- **副标题**: 18-20px, 中等
- **正文**: 16px, 常规
- **说明**: 12-14px, 常规

### 间距规范

- **页面边距**: 16px
- **组件间距**: 12px
- **内容间距**: 8px
- **按钮高度**: 44px (符合 iOS 触摸标准)

### 圆角规范

- **设备圆角**: 39px (iPhone 16 Pro)
- **卡片圆角**: 12px
- **按钮圆角**: 8px
- **输入框圆角**: 8px

## 响应式设计

### iPhone 16 Pro 适配

- **屏幕尺寸**: 393×852 points
- **安全区域**: 顶部 44px，底部 34px
- **标签栏高度**: 83px
- **导航栏高度**: 44px

### 关键适配点

1. **安全区域处理**: 所有内容避开刘海和底部指示器
2. **触摸区域**: 最小 44×44px 触摸目标
3. **圆角设计**: 39px 设备圆角模拟真机效果
4. **状态栏**: 44px 高度预留空间

## 交互设计

### 动画效果

- **页面转场**: 淡入淡出，300ms
- **模态框**: 从底部滑入，400ms
- **按钮反馈**: 缩放效果，100ms
- **加载状态**: 骨架屏和进度指示

### 微交互

- **悬停效果**: 卡片阴影变化
- **点击反馈**: 按钮缩放和颜色变化
- **状态切换**: 平滑过渡动画
- **成功反馈**: 绿色对勾动画

## 数据模型

### 用户认证模型

```javascript
{
  userId: "string",
  phone: "string",
  countryCode: "string", // 默认 "+86"
  userType: "customer" | "worker",
  authMethods: {
    password: "boolean",
    sms: "boolean",
    apple: "boolean",
    google: "boolean",
    facebook: "boolean"
  },
  profile: {
    name: "string",
    avatar: "string",
    isVerified: "boolean",
    verificationDocuments: ["string"] // 装修工认证文件
  },
  session: {
    token: "string",
    expiresAt: "timestamp",
    refreshToken: "string"
  },
  createdAt: "timestamp",
  lastLoginAt: "timestamp"
}
```

### 用户模型

```javascript
{
  userId: "string",
  userType: "customer|worker",
  profile: {
    name: "string",
    avatar: "string",
    phone: "string",
    location: { lat: number, lng: number, address: "string" }
  },
  verification: { isVerified: boolean }
}
```

### 订单模型

```javascript
{
  orderId: "string",
  customerId: "string",
  workerId: "string|null",
  status: "pending|accepted|in_progress|completed",
  location: { lat: number, lng: number, address: "string" },
  details: {
    renovationType: "kitchen|bathroom|living_room|bedroom|full_house",
    budget: "1-5w|5-10w|10-20w|20w+",
    description: "string",
    photos: ["string"]
  }
}
```

## API 集成说明

### Google Maps API

- **API 密钥**: 需要配置有效的 Google Maps API 密钥
- **所需服务**: Maps JavaScript API, Places API
- **功能**: 地图显示、地址搜索、地理编码

### 模拟数据

当前所有数据都是模拟数据，包括：

- 用户信息
- 订单列表
- 装修工信息
- 评价数据
- 作品集数据

## 部署说明

### 本地运行

1. 确保所有文件在同一目录下
2. 配置 Google Maps API 密钥
3. 使用本地服务器运行（如 Live Server）
4. 访问 index.html 查看完整原型

### 生产部署

1. 上传所有文件到 Web 服务器
2. 确保 HTTPS 协议（Google Maps 要求）
3. 配置正确的 API 密钥和域名限制
4. 测试所有页面功能

## React Native 迁移指南

### 组件映射

- **HTML 元素** → **React Native 组件**
- `<div>` → `<View>`
- `<span>`, `<p>` → `<Text>`
- `<input>` → `<TextInput>`
- `<button>` → `<TouchableOpacity>`
- `<img>` → `<Image>`

### 样式迁移

- **CSS 样式** → **StyleSheet 对象**
- **Flexbox 布局**: 直接迁移
- **绝对定位**: 需要调整
- **媒体查询**: 使用 Dimensions API

### 导航结构

```javascript
// 建议使用React Navigation
const UserStack = createBottomTabNavigator({
  Home: UserHomeScreen,
  Nearby: UserNearbyScreen,
  Profile: UserProfileScreen,
});

const WorkerStack = createBottomTabNavigator({
  Map: WorkerHomeScreen,
  Orders: WorkerOrdersScreen,
  Profile: WorkerProfileScreen,
});
```

### 地图集成

```javascript
// 使用react-native-maps
import MapView, { Marker } from 'react-native-maps';

<MapView style={styles.map} initialRegion={region} onPress={handleMapPress}>
  {orders.map(order => (
    <Marker
      key={order.id}
      coordinate={order.location}
      onPress={() => showOrderDetail(order)}
    />
  ))}
</MapView>;
```

### 状态管理建议

- **Redux Toolkit**: 全局状态管理
- **React Query**: 服务端状态管理
- **AsyncStorage**: 本地数据持久化

## 性能优化建议

### 图片优化

- 使用 WebP 格式
- 实现懒加载
- 添加占位符

### 代码优化

- 组件懒加载
- 虚拟滚动
- 防抖节流

### 网络优化

- API 请求缓存
- 离线功能支持
- 错误重试机制

## 测试建议

### 功能测试

- [ ] 所有页面正常加载
- [ ] 认证流程完整性测试
- [ ] 手机号格式验证
- [ ] 短信验证码输入和验证
- [ ] 用户类型选择功能
- [ ] 地图功能正常
- [ ] 表单提交验证
- [ ] 模态框交互
- [ ] 标签页切换

### 兼容性测试

- [ ] Safari 浏览器
- [ ] Chrome 浏览器
- [ ] 移动端 Safari
- [ ] 不同屏幕尺寸

### 用户体验测试

- [ ] 加载速度
- [ ] 动画流畅度
- [ ] 触摸响应
- [ ] 错误处理

## 后续开发建议

### 短期优化

1. 添加真实 API 接口
2. 完善认证系统后端集成
3. 实现社交登录功能
4. 添加推送通知
5. 优化地图性能

### 长期规划

1. 支付系统集成
2. 实时聊天功能
3. 评价系统完善
4. 数据分析面板

## 联系信息

如有技术问题或需要进一步说明，请联系开发团队。

---

**版本**: 1.0.0  
**最后更新**: 2024 年 1 月  
**维护团队**: 前端开发组
