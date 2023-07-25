package org.ipmes;

import org.json.JSONArray;
import org.json.JSONObject;

import java.io.BufferedReader;
import java.io.Reader;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Iterator;
import java.util.Optional;
import java.util.stream.Collectors;

public class DependencyGraph {
    ArrayList<ArrayList<Integer>> parents;
    ArrayList<ArrayList<Integer>> child;

    DependencyGraph(ArrayList<ArrayList<Integer>> parents, ArrayList<ArrayList<Integer>> child) {
        this.parents = parents;
        this.child = child;
    }

    static ArrayList<Integer> parseArrayOfEdgeId(JSONArray array) {
        ArrayList<Integer> res = new ArrayList<>();
        for (int i = 0; i < array.length(); ++i) {
            Object obj = array.get(i);
            Integer item = -1;
            if (obj instanceof Integer) {
                item = (Integer)obj;
            }
            res.add(item);
        }
        return res;
    }

    public static Optional<DependencyGraph> parse(Reader orelsReader) {
        HashMap<Integer, ArrayList<Integer>> parentsMap = new HashMap<>();
        HashMap<Integer, ArrayList<Integer>> childMap = new HashMap<>();
        try {
            BufferedReader orelsBuf = new BufferedReader(orelsReader);
            String content = orelsBuf.lines().collect(Collectors.joining());
            orelsBuf.close();

            JSONObject graphObj = new JSONObject(content);
            Iterator<String> keys = graphObj.keys();
            while (keys.hasNext()) {
                String key = (String) keys.next();
                JSONObject curObj = graphObj.getJSONObject(key);

                int id = -1;
                if (!key.equals("root"))
                    id = Integer.parseInt(key);
                id += 1;

                parentsMap.put(id, parseArrayOfEdgeId(curObj.getJSONArray("parents")));
                childMap.put(id, parseArrayOfEdgeId(curObj.getJSONArray("children")));
            }
        } catch (Exception e) {
            return Optional.empty();
        }

        int numEdges = parentsMap.size();
        ArrayList<ArrayList<Integer>> parentsList = new ArrayList<>(numEdges);
        ArrayList<ArrayList<Integer>> childsList = new ArrayList<>(numEdges);
        for (int i = 0; i < numEdges; ++i) {
            parentsList.add(i, parentsMap.get(i));
            childsList.add(i, childMap.get(i));
        }
        return Optional.of(new DependencyGraph(parentsList, childsList));
    }

    public ArrayList<Integer> getParents(Integer eid) {
        return this.parents.get(eid + 1);
    }

    public ArrayList<Integer> getChildren(Integer eid) {
        return this.child.get(eid + 1);
    }
}
