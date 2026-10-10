// Hugeicons' free set (MIT, @hugeicons/core-free-icons), under the names the UI uses.
// Add an icon by importing it from the package and giving it a name here.
import {
  AlertCircleIcon,
  Archive02Icon,
  ArrowDown01Icon,
  ArrowTurnForwardIcon,
  Attachment01Icon,
  Cancel01Icon,
  Delete02Icon,
  InboxIcon,
  Mail01Icon,
  MinusSignIcon,
  PencilEdit02Icon,
  PreferenceHorizontalIcon,
  RefreshIcon,
  ReplyAllIcon,
  ReplyIcon,
  Search01Icon,
  SentIcon,
  SidebarLeftIcon,
  SquareIcon,
  StarIcon,
  Tag01Icon,
  UserAdd01Icon,
} from "@hugeicons/core-free-icons";

/** An icon's elements as [tag, attributes] pairs, drawn on a 24px grid. */
export type IconData = readonly (readonly [string, { readonly [key: string]: string | number }])[];

export const glyphs = {
  Archive: Archive02Icon,
  ChevronDown: ArrowDown01Icon,
  CircleAlert: AlertCircleIcon,
  Forward: ArrowTurnForwardIcon,
  Inbox: InboxIcon,
  Mail: Mail01Icon,
  PanelLeft: SidebarLeftIcon,
  Paperclip: Attachment01Icon,
  Pencil: PencilEdit02Icon,
  RefreshCw: RefreshIcon,
  Reply: ReplyIcon,
  ReplyAll: ReplyAllIcon,
  Search: Search01Icon,
  Send: SentIcon,
  SlidersHorizontal: PreferenceHorizontalIcon,
  Star: StarIcon,
  /** The same star; icon.svelte fills names ending in "Filled". */
  StarFilled: StarIcon,
  Tag: Tag01Icon,
  Trash: Delete02Icon,
  UserPlus: UserAdd01Icon,
  WinMaximize: SquareIcon,
  WinMinimize: MinusSignIcon,
  X: Cancel01Icon,
} satisfies Record<string, IconData>;

export type IconName = keyof typeof glyphs;
