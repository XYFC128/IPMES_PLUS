package org.ipmes;

import java.util.ArrayList;
import java.util.Optional;

public class SharedNodeRelation {
    ArrayList<Integer>[] relation;

    SharedNodeRelation(ArrayList<Integer>[] relation) {
        this.relation = relation;
    }

    public Optional<SharedNodeRelation> build(PatternEdge e1, PatternEdge e2) {
        ArrayList<Integer>[] relation = new ArrayList[] {new ArrayList<>(), new ArrayList<>()};
        Integer[] endpoints1 = e1.getEndponts();
        Integer[] endpoints2 = e2.getEndponts();
        int count = 0;
        for (int i = 0; i < 2; ++i) {
            for (int j = 0; j < 2; ++j) {
                if (endpoints1[i].equals(endpoints2[j])) {
                    relation[i].add(j);
                    ++count;
                }
            }
        }
        if (count == 0)
            return Optional.empty();
        return Optional.of(new SharedNodeRelation(relation));
    }

    public boolean check(MatchEdge e1, MatchEdge e2) {
        Integer[] endpoints1 = e1.getEndponts();
        Integer[] endpoints2 = e2.getEndponts();
        for (int i = 0; i < 2; ++i) {
            for (int j : this.relation[i]) {
                if (endpoints1[i].equals(endpoints2[j]))
                    return false;
            }
        }
        return true;
    }
}
