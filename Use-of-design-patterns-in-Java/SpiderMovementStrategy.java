package dungeonmania.entities.enemies;

import java.util.List;

import dungeonmania.Game;
import dungeonmania.entities.Boulder;
import dungeonmania.entities.Entity;
import dungeonmania.util.Position;

public class SpiderMovementStrategy extends MovementStrategy {

    @Override
    public void move(Game game, Enemy enemy) {
        Spider spiderEnemy = (Spider) enemy;

        Position nextPos =  spiderEnemy.getMovementTrajectory().get(spiderEnemy.getNextPositionElement());
        List<Entity> entities = game.getMap().getEntities(nextPos);
        if (entities != null && entities.size() > 0 && entities.stream().anyMatch(e -> e instanceof Boulder)) {
            spiderEnemy.setForward(!spiderEnemy.getForward());
            spiderEnemy.updateNextPosition();
            spiderEnemy.updateNextPosition();
        }

        nextPos = spiderEnemy.getMovementTrajectory().get(spiderEnemy.getNextPositionElement());
        entities = game.getMap().getEntities(nextPos);
        if (entities == null || entities.size() == 0
                || entities.stream().allMatch(e -> e.canMoveOnto(game.getMap(), spiderEnemy))) {
            game.getMap().moveTo(spiderEnemy, nextPos);
            spiderEnemy.updateNextPosition();
        }
    }
}
