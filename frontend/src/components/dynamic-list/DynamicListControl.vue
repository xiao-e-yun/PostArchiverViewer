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
import { LayoutListIcon, LoaderCircle } from "lucide-vue-next";
import { dynamicListControlKey, useDynamicListControlUtils } from "./utils";
import {
  SelectValue,
  SelectContent,
  SelectTrigger,
  Select,
  SelectItem,
} from "../ui/select";
import { Button } from "../ui/button";
import { useRouter } from "vue-router";

const { total, pending, pageIndex, itemsPrePage } = inject(
  dynamicListControlKey,
)!;

const { smallMode, siblingCount } = useDynamicListControlUtils();
const postsPrePageSelectValue = computed({
  get: () => itemsPrePage.value?.toString(),
  set: (value: string) => (itemsPrePage.value = parseInt(value)),
});

const router = useRouter();
function updatePageIndex(page: number) {
  const query = { ...router.currentRoute.value.query };
  if (page !== 1) {
    query.page = page.toString();
  } else {
    delete query.page;
  }
  router.push({ query });

  pageIndex.value = page;
}
</script>

<template>
  <div class="flex flex-row-reverse justify-between gap-2 flex-nowrap">
    <div class="flex gap-2 flex-wrap justify-end">
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
        :items-per-page="itemsPrePage"
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

    <div v-if="pending" class="w-0 my-auto -z-10">
      <LoaderCircle class="animate-spin h-auto aspect-square" :size="32" />
    </div>
  </div>
</template>
