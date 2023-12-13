package org.ipmes.pattern;

import org.json.JSONArray;
import org.json.JSONObject;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;

import static java.lang.Math.max;

public class PatternParser {
    public static Pattern parse(String patternFile) throws IOException {
        String content = new String(Files.readAllBytes(Paths.get(patternFile)));
        JSONObject patternJSON = new JSONObject(content);
        boolean useRegex = patternJSON.getBoolean("UseRegex");

        JSONArray eventsArray = patternJSON.getJSONArray("Events");
        PatternGraph patternGraph = parsePatternGraph(eventsArray);
        TemporalRelation relation = parseTemporalRelation(eventsArray);

        Pattern pattern = new Pattern(patternGraph, relation);
        pattern.useRegex = useRegex;
        return pattern;
    }

    static PatternGraph parsePatternGraph(JSONArray eventsArray) {
        ArrayList<PatternEdge> edges = new ArrayList<>();
        HashMap<Integer, PatternNode> nodesMap = new HashMap<>();
        int max_node = 0;
        for (Object obj : eventsArray) {
            JSONObject eventObj = (JSONObject) obj;

            String signature = eventObj.getString("Signature");
            int edgeId       = eventObj.getInt("ID");
            int startId      = eventObj.getInt("SubjectID");
            int endId        = eventObj.getInt("ObjectID");

            max_node = max(max(startId, endId), max_node);

            nodesMap.putIfAbsent(startId, new PatternNode(startId, ""));
            nodesMap.putIfAbsent(endId, new PatternNode(endId, ""));

            edges.add(new PatternEdge(edgeId, signature, nodesMap.get(startId), nodesMap.get(endId), true));
        }

        ArrayList<PatternNode> nodes = new ArrayList<>();
        for (int i = 0; i <= max_node; i++) {
            nodes.add(nodesMap.get(i));
        }

        return new PatternGraph(nodes, edges);
    }

    static TemporalRelation parseTemporalRelation(JSONArray eventsArray) {
        HashMap<Integer, ArrayList<Integer>> parentsMap = new HashMap<>();
        HashMap<Integer, ArrayList<Integer>> childMap = new HashMap<>();

        for (Object obj : eventsArray) {
            JSONObject eventObj = (JSONObject) obj;
            JSONArray parentsArray = eventObj.getJSONArray("Parents");
            Integer myId = eventObj.getInt("ID");

            parentsMap.putIfAbsent(myId, new ArrayList<>());
            for (Object parent : parentsArray) {
                Integer parentID = (Integer) parent;
                childMap.putIfAbsent(parentID, new ArrayList<>());

                parentsMap.get(myId).add(parentID);
                childMap.get(parentID).add(myId);
            }
            if (parentsArray.isEmpty()) {
                childMap.putIfAbsent(-1, new ArrayList<>());
                parentsMap.get(myId).add(-1);
                childMap.get(-1).add(myId);
            }
        }

        int numEdges = eventsArray.length() + 1;
        ArrayList<ArrayList<Integer>> parentsList = new ArrayList<>(numEdges);
        ArrayList<ArrayList<Integer>> childrenList = new ArrayList<>(numEdges);
        for (int i = 0; i < numEdges; ++i) {
            int internalId = i - 1;
            if (parentsMap.containsKey(internalId))
                parentsList.add(i, parentsMap.get(internalId));
            else
                parentsList.add(i, new ArrayList<>());

            if (childMap.containsKey(internalId))
                childrenList.add(i, childMap.get(internalId));
            else
                childrenList.add(i, new ArrayList<>());
        }

        return new TemporalRelation(parentsList, childrenList);
    }
}
