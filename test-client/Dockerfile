FROM node:alpine AS build

RUN apk --no-cache add git

WORKDIR /app

COPY package.json ./
COPY yarn.lock ./
RUN yarn
COPY . .
RUN yarn build

FROM nginx:alpine

COPY --from=build /app/dist /usr/share/nginx/html
EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
