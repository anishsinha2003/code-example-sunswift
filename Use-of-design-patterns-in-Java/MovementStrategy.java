package dungeonmania.entities.enemies;

import java.util.List;
import java.util.Random;
import java.util.stream.Collectors;

import dungeonmania.Game;
import dungeonmania.map.GameMap;
import dungeonmania.util.Direction;
import dungeonmania.util.Position;

public abstract class MovementStrategy {
    public abstract void move(Game game, Enemy enemy);

    // this function is specifically for Mercernary and Zombietoast Movement. If the player is affected by Invincibility
    // this rmemoves the repeated code which was apparent in both implementation. Now in the MercernaryMovement
    // strategy,and ZombiToastMovementStrategy, this function is called, insead
    // of doing the whole implementation twice
    public Position ifEffectedByInvinsibility(Game game, Enemy enemy) {
        GameMap map = game.getMap();
        Position plrDiff = Position.calculatePositionBetween(map.getPlayer().getPosition(), enemy.getPosition());

        Position moveX = (plrDiff.getX() >= 0) ? Position.translateBy(enemy.getPosition(), Direction.RIGHT)
                : Position.translateBy(enemy.getPosition(), Direction.LEFT);
        Position moveY = (plrDiff.getY() >= 0) ? Position.translateBy(enemy.getPosition(), Direction.UP)
                : Position.translateBy(enemy.getPosition(), Direction.DOWN);
        Position offset = enemy.getPosition();

        // changed this if statements to get rid of repeated cases
        if (plrDiff.getY() == 0) {
            offset = map.canMoveTo(enemy, moveX) ? moveX : offset;
        } else if (plrDiff.getX() == 0) {
            offset = map.canMoveTo(enemy, moveY) ? moveY : offset;
        } else if (Math.abs(plrDiff.getX()) >= Math.abs(plrDiff.getY())) {
            offset = map.canMoveTo(enemy, moveX) ? moveX : (map.canMoveTo(enemy, moveY) ? moveY : offset);
        } else {
            offset = map.canMoveTo(enemy, moveY) ? moveY : (map.canMoveTo(enemy, moveX) ? moveX : offset);
        }
        return offset;
    }

    // this function also is specifically for Mercernary and Zombietoast Movement.
    // this removes the repeated code which was apparent in both implementation. Now in the MercernaryMovement
    // strategy,and ZombiToastMovementStrategy, this function is called, insead
    // of doing the whole implementation twice
    public Position moveRandom(Game game, Enemy enemy) {
        Position nextPos;
        GameMap map = game.getMap();
        Random randGen = new Random();

        // got rid of code which violated LoD by adding a new method in entity that gets neighbours
        List<Position> pos = enemy.getNeighbours();
        pos = pos.stream().filter(p -> map.canMoveTo(enemy, p)).collect(Collectors.toList());
        if (pos.size() == 0) {
            nextPos = enemy.getPosition();
        } else {
            nextPos = pos.get(randGen.nextInt(pos.size()));
        }
        return nextPos;
    }
}
