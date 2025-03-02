<script setup lang="ts">
import { computed, inject } from "vue";
import {
  Pagination,
  PaginationEllipsis,
  PaginationFirst,
  PaginationLast,
  PaginationList,
  PaginationListItem,
  PaginationNext,
  PaginationPrev,
} from "../ui/pagination";
import { LayoutListIcon } from "lucide-vue-next";
import { postListControlKey, usePostListControlUtils } from "./utils";
import {
  SelectValue,
  SelectContent,
  SelectTrigger,
  Select,
  SelectItem,
} from "../ui/select";
import { Button } from "../ui/button";
import { useRouter } from "vue-router";

const { total, pageIndex, postsPrePage } = inject(postListControlKey)!;

const { smallMode, siblingCount } = usePostListControlUtils();
const postsPrePageSelectValue = computed({
  get: () => postsPrePage.value?.toString(),
  set: (value: string) => (postsPrePage.value = parseInt(value)),
});

const router = useRouter();
function updatePageIndex(page: number) {
  const query = { ...router.currentRoute.value.query };
  if (page !== 1) query.page = page.toString();
  router.push({ query });

  pageIndex.value = page;
}
</script>

<template>
  <div class="flex justify-end gap-2 flex-wrap">
    <Select v-model="postsPrePageSelectValue">
      <SelectTrigger class="w-32">
        <LayoutListIcon />
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="20">20</SelectItem>
        <SelectItem value="50">50</SelectItem>
        <SelectItem value="100">100</SelectItem>
      </SelectContent>
    </Select>

    <Pagination
      v-slot="{ page }"
      :page="pageIndex"
      :total="total"
      :sibling-count="siblingCount"
      :show-edges="!smallMode"
      :items-per-page="postsPrePage"
      @update:page="updatePageIndex"
    >
      <PaginationList v-slot="{ items }" class="flex items-center gap-1">
        <PaginationFirst />
        <PaginationPrev />

        <template v-for="(item, index) in items">
          <PaginationListItem
            v-if="item.type === 'page'"
            :key="index"
            :value="item.value"
            as-child
          >
            <Button
              class="w-10 h-10 p-0"
              :variant="item.value === page ? 'default' : 'outline'"
            >
              {{ item.value }}
            </Button>
          </PaginationListItem>
          <PaginationEllipsis v-else :key="item.type" :index="index" />
        </template>

        <PaginationNext />
        <PaginationLast />
      </PaginationList>
    </Pagination>
  </div>
</template>
