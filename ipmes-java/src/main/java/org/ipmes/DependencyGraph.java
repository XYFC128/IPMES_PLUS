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
    ArrayList<ArrayList<Integer>> childs;

    DependencyGraph(ArrayList<ArrayList<Integer>> parents, ArrayList<ArrayList<Integer>> childs) {
        this.parents = parents;
        this.childs = childs;
    }

    public static Optional<DependencyGraph> parse(Reader orelsReader) {
        HashMap<Integer, ArrayList<Integer>> parentsMap = new HashMap<>();
        HashMap<Integer, ArrayList<Integer>> childsMap = new HashMap<>();
        try {
            BufferedReader orelsBuf = new BufferedReader(orelsReader);
            String content = orelsBuf.lines().collect(Collectors.joining());
            orelsBuf.close();

            JSONObject graphObj = new JSONObject(content);
            Iterator<String> keys = graphObj.keys();
            while (keys.hasNext()) {
                String key = (String) keys.next();
                JSONObject curObj = graphObj.getJSONObject(key);

                Integer id = -1;
                if (!key.equals("root"))
                    id = Integer.parseInt(key);
                id += 1;

                JSONArray curParents = curObj.getJSONArray("parents");
                parentsMap.put(id, new ArrayList<>());
                for (int i = 0; i < curParents.length(); ++i) {
                    Object tmp = curParents.get(i);
                    Integer p = -1;
                    if (tmp instanceof Integer) {
                        p = (Integer)tmp;
                    }
                    parentsMap.get(id).add(p);
                }

                JSONArray curChildren = curObj.getJSONArray("children");
                childsMap.put(id, new ArrayList<>());
                for (int i = 0; i < curChildren.length(); ++i) {
                    Object tmp = curChildren.get(i);
                    Integer c = -1;
                    if (tmp instanceof Integer) {
                        c = (Integer)tmp;
                    }
                    childsMap.get(id).add(c);
                }
            }
        } catch (Exception e) {
            return Optional.empty();
        }

        Integer numEdges = parentsMap.size();
        ArrayList<ArrayList<Integer>> parentsList = new ArrayList<>(numEdges);
        ArrayList<ArrayList<Integer>> childsList = new ArrayList<>(numEdges);
        for (int i = 0; i < numEdges; ++i) {
            parentsList.add(i, parentsMap.get(i));
            childsList.add(i, childsMap.get(i));
        }
        return Optional.of(new DependencyGraph(parentsList, childsList));
    }

    public ArrayList<Integer> getParents(Integer eid) {
        return this.parents.get(eid + 1);
    }

    public ArrayList<Integer> getChildrens(Integer eid) {
        return this.childs.get(eid + 1);
    }
}
