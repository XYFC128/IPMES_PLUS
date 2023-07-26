package org.ipmes;

public class TTPGenerator {
    public static String genTTP11Nodes() {
        String nodes = "{\"node\":{\"type\":\"node\",\"id\":\"49768\",\"labels\":[\"VERTEX\"],\"properties\":{\"gid\":\"0\",\"euid\":\"0\",\"children pid namespace\":\"-1\",\"start time\":\"1640055482.072\",\"pid\":\"6161\",\"source\":\"syscall\",\"type\":\"Process\",\"net namespace\":\"-1\",\"ppid\":\"6160\",\"ipc namespace\":\"-1\",\"pid namespace\":\"-1\",\"uid\":\"0\",\"cwd\":\"/root\",\"egid\":\"0\",\"mount namespace\":\"-1\",\"subtype\":\"process\",\"command line\":\"/bin/bash /home/admin/.mozilla/firefox/hello.sh\",\"name\":\"hello.sh\",\"hash\":\"62a7d76166b24814dddc9eae4824a229\",\"user namespace\":\"-1\"}}}\n" +
                "{\"node\":{\"type\":\"node\",\"id\":\"275832\",\"labels\":[\"VERTEX\"],\"properties\":{\"gid\":\"0\",\"euid\":\"0\",\"children pid namespace\":\"4026531836\",\"start time\":\"1640056855.788\",\"ns pid\":\"13939\",\"pid\":\"13939\",\"source\":\"syscall\",\"type\":\"Process\",\"net namespace\":\"4026531992\",\"ipc namespace\":\"4026531839\",\"ppid\":\"6161\",\"pid namespace\":\"4026531836\",\"cwd\":\"/root\",\"uid\":\"0\",\"egid\":\"0\",\"mount namespace\":\"4026531840\",\"subtype\":\"process\",\"command line\":\"/bin/bash /home/admin/.mozilla/firefox/hello.sh\",\"name\":\"hello.sh\",\"hash\":\"6cf7e2d84d0879f547b338544d0b882d\",\"user namespace\":\"4026531837\"}}}\n" +
                "{\"node\":{\"type\":\"node\",\"id\":\"275834\",\"labels\":[\"VERTEX\"],\"properties\":{\"gid\":\"0\",\"euid\":\"0\",\"children pid namespace\":\"4026531836\",\"start time\":\"1640056855.788\",\"ns pid\":\"13939\",\"pid\":\"13939\",\"source\":\"syscall\",\"type\":\"Process\",\"net namespace\":\"4026531992\",\"ipc namespace\":\"4026531839\",\"ppid\":\"6161\",\"pid namespace\":\"4026531836\",\"cwd\":\"/root\",\"uid\":\"0\",\"egid\":\"0\",\"mount namespace\":\"4026531840\",\"subtype\":\"process\",\"command line\":\"journalctl --vacuum-time=1s\",\"name\":\"journalctl\",\"hash\":\"1ef9c631e77d04144add51fdce9a3cf0\",\"user namespace\":\"4026531837\"}}}\n" +
                "{\"node\":{\"type\":\"node\",\"id\":\"66856\",\"labels\":[\"VERTEX\"],\"properties\":{\"inode\":\"92283629\",\"path\":\"/var/log/journal\",\"subtype\":\"directory\",\"permissions\":\"2755\",\"epoch\":\"0\",\"source\":\"syscall\",\"type\":\"Artifact\",\"version\":\"0\",\"hash\":\"0c6bbf366e48b1aa4e29db339327a2f9\"}}}\n" +
                "{\"node\":{\"type\":\"node\",\"id\":\"275835\",\"labels\":[\"VERTEX\"],\"properties\":{\"inode\":\"262755\",\"path\":\"/usr/bin/journalctl\",\"subtype\":\"file\",\"permissions\":\"0755\",\"epoch\":\"0\",\"source\":\"syscall\",\"type\":\"Artifact\",\"version\":\"0\",\"hash\":\"0db4a541cffffcc5b1e1fa9773d3b709\"}}}\n";
        return nodes;
    }

    public static String genTTP11Edges() {
        String edges = "{\"edge\":{\"id\":\"707540\",\"type\":\"relationship\",\"label\":\"newnewEDGE\",\"properties\":{\"event id\":\"883152\",\"flags\":\"CLONE_CHILD_CLEARTID|SIGCHLD|CLONE_CHILD_SETTID\",\"time\":\"1640056855.788\",\"source\":\"syscall\",\"type\":\"WasTriggeredBy\",\"operation\":\"fork\",\"hash\":\"8019244e0fd3c9727d084ad6001c5d6f\"},\"start\":{\"id\":\"49768\",\"labels\":[\"VERTEX\"]},\"end\":{\"id\":\"275832\",\"labels\":[\"VERTEX\"]}}}\n" +
                "{\"edge\":{\"id\":\"707541\",\"type\":\"relationship\",\"label\":\"newnewEDGE\",\"properties\":{\"event id\":\"883158\",\"time\":\"1640056855.788\",\"source\":\"syscall\",\"type\":\"WasTriggeredBy\",\"operation\":\"execve\",\"hash\":\"dcc88198602db9a3813f21f74ad36304\"},\"start\":{\"id\":\"275832\",\"labels\":[\"VERTEX\"]},\"end\":{\"id\":\"275834\",\"labels\":[\"VERTEX\"]}}}\n" +
                "{\"edge\":{\"id\":\"555023\",\"type\":\"relationship\",\"label\":\"newnewEDGE\",\"properties\":{\"event id\":\"883158\",\"time\":\"1640056855.788\",\"source\":\"syscall\",\"type\":\"Used\",\"operation\":\"load\",\"hash\":\"b2aa281875d9ed8934eb8f3554366fee\"},\"start\":{\"id\":\"275835\",\"labels\":[\"VERTEX\"]},\"end\":{\"id\":\"275834\",\"labels\":[\"VERTEX\"]}}}\n" +
                "{\"edge\":{\"id\":\"554975\",\"type\":\"relationship\",\"label\":\"newnewEDGE\",\"properties\":{\"event id\":\"883485\",\"flags\":\"O_RDONLY\",\"time\":\"1640056855.812\",\"source\":\"syscall\",\"type\":\"Used\",\"operation\":\"open\",\"hash\":\"912f968d83ab5d94aaf8b8ee5057890d\"},\"start\":{\"id\":\"66856\",\"labels\":[\"VERTEX\"]},\"end\":{\"id\":\"275834\",\"labels\":[\"VERTEX\"]}}}\n";
        return edges;
    }

    public static String genTTP11Orels() {
        String orels = "{\"0\":{\"parents\":[\"root\"],\"children\":[1]},\"1\":{\"parents\":[0],\"children\":[2]},\"2\":{\"parents\":[1],\"children\":[3]},\"3\":{\"parents\":[2],\"children\":[]},\"root\":{\"parents\":[],\"children\":[0]}}";
        return orels;
    }
}
