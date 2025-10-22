<script setup lang="tsx">
import { computed, inject, ref } from "vue";
import { postKey } from "./utils";
import type { Comment } from "@api/Comment";
import Separator from "../ui/separator/Separator.vue";
import { Button } from "../ui/button";
import { createReusableTemplate } from "@vueuse/core";
import { Badge } from "../ui/badge";
import Collapsible from "../ui/collapsible/Collapsible.vue";
import { CollapsibleContent, CollapsibleTrigger } from "../ui/collapsible";
import { ChevronDown } from "lucide-vue-next";

const { post } = inject(postKey)!;
const comments = computed(() => (post.value?.comments as Comment[]) ?? []);

const showCount = ref(3);

const [DefineComment, Comment] = createReusableTemplate<{
  comment: Comment;
}>();
</script>

<template>
  <DefineComment v-slot="{ comment }">
    <div class="flex flex-col gap-1 my-2">
      <Badge variant="secondary" class="mr-auto rounded-md px-2">
        {{ comment.user }}
      </Badge>
      <p>{{ comment.text }}</p>
      <Collapsible>
        <CollapsibleTrigger
          v-if="comment.replies?.length"
          class="text-sm mt-1 text-left group"
        >
          <ChevronDown
            class="inline transition-transform group-data-[state=open]:rotate-180"
            :size="20"
          />
          {{ comment.replies.length }} Replies
        </CollapsibleTrigger>
        <CollapsibleContent class="ml-10">
          <Comment
            v-for="(reply, i) in comment.replies"
            :key="i"
            :comment="reply"
          />
        </CollapsibleContent>
      </Collapsible>
    </div>
  </DefineComment>

  <div v-if="comments.length" class="flex flex-col gap-4">
    <Separator label="Comments" class="my-4" />
    <Comment
      v-for="(comment, i) in comments.slice(0, showCount)"
      :key="i"
      :comment="comment"
    />
    <div
      v-if="comments.length > showCount"
      class="flex w-full items-center gap-2 mt-2"
    >
      <Separator class="w-full flex-0" />
      <Button variant="ghost" class="text-xs px-2 py-0" @click="showCount += 5">
        Show comments ({{ comments.length - showCount }} more)
      </Button>
      <Separator class="w-full flex-0" />
    </div>
  </div>
</template>
